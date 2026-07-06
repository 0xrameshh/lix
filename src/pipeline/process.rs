use std::path::Path;

use serde_json::Value;

use crate::cleaner::{CleanReport, Cleaner};
use crate::error::{Result, TraceForgeError};
use crate::models::TraceType;
use crate::parser::{self, RawEvent};
use crate::providers::{self, NormalizedSession};
use crate::report::{FileReport, FileStatus};
use crate::sqlite;
use crate::tools::{self, provider_builtins};

use super::dedup::DedupTracker;

pub fn process_file(
    path: &Path,
    cleaner: &Cleaner,
    opts: &super::PipelineOpts,
    dedup: &DedupTracker,
) -> Result<Vec<(Option<Vec<u8>>, FileReport)>> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let empty_events = Vec::new();

    let (session_events, use_streaming): (&[Vec<RawEvent>], bool) = match ext {
        "db" | "vscdb" => (&read_sqlite_sessions(path)?, false),
        _ => {
            let peek = parser::peek_events(path, 10)?;
            if peek.is_empty() {
                return Ok(vec![(
                    None,
                    FileReport {
                        source: path.display().to_string(),
                        trace_type: "unknown".into(),
                        status: FileStatus::Empty,
                        messages: 0,
                        tool_calls: 0,
                        dropped_reason: Some("empty".into()),
                        clean: CleanReport::default(),
                    },
                )]);
            }
            (&empty_events, true)
        }
    };

    let mut results: Vec<(Option<Vec<u8>>, FileReport)> = Vec::new();
    let mut had_unsupported = false;

    if use_streaming {
        let peek = parser::peek_events(path, 100)?;
        let tt = providers::detect(&peek);
        if tt.as_str() == "unknown" {
            had_unsupported = true;
        } else if opts
            .model_filter
            .as_ref()
            .is_none_or(|f| model_in_events(&peek, f))
        {
            process_session(
                path,
                cleaner,
                opts,
                dedup,
                &mut results,
                &mut had_unsupported,
                tt,
                |p| p.normalize_file(path),
            )?;
        }
    } else {
        for events in session_events {
            let tt = providers::detect(events);
            if tt.as_str() == "unknown" {
                had_unsupported = true;
                continue;
            }
            if opts
                .model_filter
                .as_ref()
                .is_some_and(|f| !model_in_events(events, f))
            {
                continue;
            }
            process_session(
                path,
                cleaner,
                opts,
                dedup,
                &mut results,
                &mut had_unsupported,
                tt,
                |p| p.normalize(path, events.to_vec()),
            )?;
        }
    }

    if results.is_empty() {
        let (status, reason) = if had_unsupported {
            (
                FileStatus::Unsupported,
                Some("unsupported trace format".into()),
            )
        } else {
            (FileStatus::Empty, Some("no valid sessions".into()))
        };
        results.push((
            None,
            FileReport {
                source: path.display().to_string(),
                trace_type: "unknown".into(),
                status,
                messages: 0,
                tool_calls: 0,
                dropped_reason: reason,
                clean: CleanReport::default(),
            },
        ));
    }

    Ok(results)
}

#[allow(clippy::too_many_arguments)]
fn process_session(
    path: &Path,
    cleaner: &Cleaner,
    opts: &super::PipelineOpts,
    dedup: &DedupTracker,
    results: &mut Vec<(Option<Vec<u8>>, FileReport)>,
    had_unsupported: &mut bool,
    tt: TraceType,
    normalize: impl Fn(&dyn providers::Provider) -> Result<NormalizedSession>,
) -> Result<()> {
    let builtins = provider_builtins(tt);
    let provider = match providers::for_type(tt) {
        Ok(p) => p,
        Err(TraceForgeError::UnsupportedProvider(_)) => {
            *had_unsupported = true;
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    let NormalizedSession {
        steps,
        metadata,
        tool_names,
        argument_samples,
        ..
    } = normalize(provider.as_ref())?;

    if !metadata.session_id.is_empty() && !dedup.try_insert(&metadata.session_id) {
        results.push((
            None,
            FileReport {
                source: path.display().to_string(),
                trace_type: tt.as_str().to_string(),
                status: FileStatus::DroppedDuplicate,
                messages: 0,
                tool_calls: 0,
                dropped_reason: Some(format!("duplicate session_id: {}", metadata.session_id)),
                clean: CleanReport::default(),
            },
        ));
        return Ok(());
    }

    let mut steps = steps;
    let mut metadata = metadata;
    let mut clean_report = CleanReport::default();
    if opts.clean {
        cleaner.clean_metadata(&mut metadata, &mut clean_report);
        for step in &mut steps {
            cleaner.clean_step(step, &mut clean_report);
        }
    }

    let tools = tools::build_tools(&builtins, &tool_names, &argument_samples);

    let msg_count = steps
        .iter()
        .filter(|s| {
            matches!(
                s,
                crate::models::Step::User { .. }
                    | crate::models::Step::LlmOnly { .. }
                    | crate::models::Step::Thought { .. }
                    | crate::models::Step::AssistantText { .. }
            )
        })
        .count();
    let tc_count = steps
        .iter()
        .filter(|s| matches!(s, crate::models::Step::ToolCall { .. }))
        .count();

    let session = providers::to_example::to_training_example(
        NormalizedSession {
            trace_type: tt,
            steps,
            metadata,
            tool_names,
            argument_samples,
        },
        tools,
        opts.drop_incomplete,
    );

    match session {
        Some(ex) => {
            let mut buf = Vec::new();
            serde_json::to_writer(&mut buf, &ex)?;
            results.push((
                Some(buf),
                FileReport {
                    source: path.display().to_string(),
                    trace_type: tt.as_str().to_string(),
                    status: FileStatus::Ok,
                    messages: msg_count,
                    tool_calls: tc_count,
                    dropped_reason: None,
                    clean: clean_report,
                },
            ));
        }
        None if opts.drop_incomplete => {}
        None => {
            return Err(TraceForgeError::IncompleteTrace {
                path: path.display().to_string(),
            })
        }
    }

    Ok(())
}

fn read_sqlite_sessions(path: &Path) -> Result<Vec<Vec<RawEvent>>> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "db" => sqlite::read_hermes_state_db(path),
        "vscdb" => sqlite::read_cursor_state_vscdb(path),
        _ => Ok(vec![]),
    }
}

fn model_in_events(events: &[RawEvent], filter: &str) -> bool {
    events.iter().any(|ev| {
        ev.message
            .as_ref()
            .and_then(|m| m.as_object())
            .and_then(|o| o.get("model"))
            .and_then(Value::as_str)
            .is_some_and(|m| m.contains(filter))
    })
}
