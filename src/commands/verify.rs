use std::path::{Path, PathBuf};

use crate::{run_pipeline, PipelineOpts};

pub fn handle_verify(
    input: PathBuf,
    golden: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut should_fail = false;
    let files = crate::FileSet::from_path(&input, &["jsonl", "json", "db", "vscdb"])?;
    let out = std::env::temp_dir().join("tf_verify_output.jsonl");

    for f in &files.files {
        let stem = f
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let golden_path = match &golden {
            Some(g) => {
                let mut p = g.clone();
                if g.is_dir() {
                    p.push(format!("{stem}.jsonl"));
                }
                p
            }
            None => Path::new("tests/goldens").join(format!("{stem}.jsonl")),
        };

        if !golden_path.exists() {
            eprintln!("[{stem:40}] SKIP  (no golden at {})", golden_path.display());
            continue;
        }

        let single = PathBuf::from(f);
        match run_pipeline(
            &single,
            &out,
            PipelineOpts {
                clean: false,
                drop_incomplete: false,
                ..Default::default()
            },
        ) {
            Ok(rep) if rep.rows_written == 0 => {
                eprintln!("[{stem:40}] FAIL  (no output rows)");
                should_fail = true;
            }
            Ok(_) => {
                let actual = std::fs::read(&out).unwrap_or_default();
                let golden_data = std::fs::read(&golden_path).unwrap_or_default();
                if actual == golden_data {
                    eprintln!("[{stem:40}] OK");
                } else {
                    let actual_lines = actual.split(|&b| b == b'\n').count();
                    let golden_lines = golden_data.split(|&b| b == b'\n').count();
                    eprintln!(
                        "[{stem:40}] DIFF  (tf:{}b/{}L, golden:{}b/{}L)",
                        actual.len(),
                        actual_lines,
                        golden_data.len(),
                        golden_lines
                    );
                    should_fail = true;
                }
            }
            Err(e) => {
                eprintln!("[{stem:40}] FAIL  ({e})");
                should_fail = true;
            }
        }
    }

    if should_fail {
        std::process::exit(1);
    }
    Ok(())
}
