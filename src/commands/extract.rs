use std::path::PathBuf;

use crate::{
    report::{FileReport, FileStatus},
    run_pipeline_files, FileSet, PipelineOpts, TraceForgeError,
};

pub(crate) fn resolve_files(
    input: &Option<PathBuf>,
) -> Result<FileSet, Box<dyn std::error::Error>> {
    match input {
        Some(path) => Ok(FileSet::from_path(path, &["jsonl", "json", "db", "vscdb"])?),
        None => {
            let traces = FileSet::scan_common();
            Ok(FileSet { files: traces })
        }
    }
}

fn print_file_report(fr: &FileReport) {
    let status = match fr.status {
        FileStatus::Ok => "OK",
        FileStatus::DroppedIncomplete => "DROP(incomplete)",
        FileStatus::DroppedSynthetic => "DROP(synthetic)",
        FileStatus::DroppedModelFilter => "DROP(model)",
        FileStatus::DroppedDuplicate => "DROP(duplicate)",
        FileStatus::Empty => "EMPTY",
        FileStatus::Unsupported => "UNSUPPORTED",
        FileStatus::Error => "ERROR",
    };
    let path = std::path::Path::new(&fr.source);
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();
    let provider = &fr.trace_type;
    eprintln!("  [{status:17}] {provider:14} {name}");
}

fn eprint_help_no_files(input: &str) {
    if input.is_empty() {
        eprintln!("No trace files found in common locations (~/.claude/logs, etc.).");
        eprintln!("Try:  lix find");
        eprintln!("Or:   lix extract <path-to-trace-file>");
    } else {
        eprintln!("No trace files found at: {input}");
        eprintln!("Try:  lix find");
    }
}

#[allow(clippy::too_many_arguments)]
pub fn handle_extract(
    input: Option<PathBuf>,
    out: PathBuf,
    model: Option<String>,
    no_clean: bool,
    keep_incomplete: bool,
    concurrency: Option<usize>,
    report: Option<PathBuf>,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let opts = PipelineOpts {
        clean: !no_clean,
        drop_incomplete: !keep_incomplete && !no_clean,
        model_filter: model,
        concurrency,
    };
    let files = resolve_files(&input)?;
    let rep = run_pipeline_files(files, &out, opts)?;
    if !quiet {
        eprintln!("Read {} files:", rep.files_total);
        for fr in &rep.files {
            print_file_report(fr);
        }
        eprintln!();
        eprintln!(
            "Summary: {} ok, {} dropped, {} errors | {} rows written | {} replacements in {:.1}s",
            rep.files_ok,
            rep.files_dropped,
            rep.files_errored,
            rep.rows_written,
            rep.total_replacements,
            rep.elapsed_secs
        );
        if rep.files_total == 0 {
            let input_label = input
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            eprint_help_no_files(&input_label);
        }
    }
    if let Some(rp) = report {
        let file = std::fs::File::create(&rp).map_err(|e| TraceForgeError::Io {
            path: rp.display().to_string(),
            source: e,
        })?;
        serde_json::to_writer_pretty(file, &rep)?;
    }
    Ok(())
}
