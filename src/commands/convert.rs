use std::path::PathBuf;

use crate::commands::extract::resolve_files;
use crate::{run_pipeline_files, PipelineOpts, TraceForgeError};

pub fn handle_convert(
    input: Option<PathBuf>,
    out: PathBuf,
    model: Option<String>,
    concurrency: Option<usize>,
    report: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let opts = PipelineOpts {
        clean: false,
        drop_incomplete: true,
        model_filter: model,
        concurrency,
    };
    let files = resolve_files(&input)?;
    let rep = run_pipeline_files(files, &out, opts)?;
    eprintln!(
        "Converted {} sessions to {} ({} rows, {:.1}s)",
        rep.files_total,
        out.display(),
        rep.rows_written,
        rep.elapsed_secs
    );
    if rep.files_total == 0 {
        let input_label = input
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        eprintln!("No trace files found at: {input_label}");
        eprintln!("Try:  lix find");
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
