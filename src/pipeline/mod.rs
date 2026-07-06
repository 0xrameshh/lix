use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

use rayon::prelude::*;

use crate::cleaner::CleanReport;
use crate::discover::FileSet;
use crate::error::{Result, TraceForgeError};
use crate::report::{ExtractionReport, FileReport};
use crate::writer::JsonlSink;
use crate::Cleaner;

mod aggregate;
mod dedup;
mod process;

use aggregate::FileAggregate;
use dedup::DedupTracker;
use process::process_file;

pub struct PipelineOpts {
    pub clean: bool,
    pub drop_incomplete: bool,
    pub model_filter: Option<String>,
    pub concurrency: Option<usize>,
}

impl Default for PipelineOpts {
    fn default() -> Self {
        Self {
            clean: true,
            drop_incomplete: true,
            model_filter: None,
            concurrency: None,
        }
    }
}

type ProcessResult = Result<Vec<(Option<Vec<u8>>, FileReport)>>;

/// Convenience: discover files from a path, then run the pipeline.
pub fn run_pipeline(input: &Path, output: &Path, opts: PipelineOpts) -> Result<ExtractionReport> {
    let files = FileSet::from_path(input, &["jsonl", "json", "db", "vscdb"])?;
    run_pipeline_files(files, output, opts)
}

/// Run the pipeline on a [`FileSet`] of trace files.
pub fn run_pipeline_files(
    files: FileSet,
    output: &Path,
    opts: PipelineOpts,
) -> Result<ExtractionReport> {
    let start = Instant::now();

    if files.is_empty() {
        return Ok(ExtractionReport::default());
    }

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(opts.concurrency.unwrap_or_else(rayon::current_num_threads))
        .build()
        .map_err(|e| TraceForgeError::Io {
            path: "rayon pool".into(),
            source: std::io::Error::other(e),
        })?;

    let mut sink = JsonlSink::open(output)?;

    let cleaner = Cleaner::new();
    let dedup = DedupTracker::new();

    let results: Vec<ProcessResult> = pool.install(|| {
        files
            .files
            .par_iter()
            .map(|file_path| process_file(file_path, &cleaner, &opts, &dedup))
            .collect()
    });

    let mut aggregate: HashMap<String, FileAggregate> = HashMap::new();
    let mut rows_written = 0usize;

    for (file_path, result) in files.files.iter().zip(results) {
        let agg_key = file_path.display().to_string();
        match result {
            Ok(sessions) => {
                for (bytes_opt, file_report) in sessions {
                    if let Some(bytes) = bytes_opt {
                        sink.write_row(&bytes)?;
                        rows_written += 1;
                    }
                    aggregate
                        .entry(agg_key.clone())
                        .or_default()
                        .record(&file_report);
                }
            }
            Err(e) => {
                tracing::warn!("error processing file: {e}");
                aggregate.entry(agg_key.clone()).or_default().errors += 1;
            }
        }
    }

    let mut report = ExtractionReport {
        rows_written,
        ..Default::default()
    };
    for (source, agg) in &aggregate {
        report.record_file(FileReport {
            source: source.clone(),
            trace_type: agg.trace_type.clone(),
            status: agg.final_status(),
            messages: agg.messages,
            tool_calls: agg.tool_calls,
            dropped_reason: if agg.dropped > 0 {
                Some(format!(
                    "{} ok, {} dropped, {} errors",
                    agg.ok, agg.dropped, agg.errors
                ))
            } else {
                None
            },
            clean: CleanReport {
                total_replacements: agg.replacements,
                ..CleanReport::default()
            },
        });
    }

    sink.flush()?;
    report.elapsed_secs = start.elapsed().as_secs_f64();
    Ok(report)
}
