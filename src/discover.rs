use std::path::{Path, PathBuf};

use crate::error::{Result, TraceForgeError};

#[derive(Debug, Clone)]
pub struct FileSet {
    pub files: Vec<PathBuf>,
}

impl FileSet {
    /// Common directories (relative to ~) where agent tools store session traces.
    pub const COMMON_TRACE_DIRS: &'static [&'static str] = &[
        ".claude/sessions",
        ".claude/transcripts",
        ".claude/logs",
        ".cursor/logs",
        ".codex/logs",
        ".codex/sessions",
        ".pi/logs",
        ".factory/droids",
        ".hermes/sessions",
        ".hermes/logs",
        ".openclaude/transcripts",
        ".openclaude/sessions",
        ".openclaw/transcripts",
        ".openclaw/sessions",
    ];

    /// Single common trace files (relative to ~) to check directly.
    pub const COMMON_TRACE_FILES: &'static [&'static str] = &[
        ".claude/history.jsonl",
        ".hermes/state.db",
        "Library/Application Support/Cursor/User/globalStorage/state.vscdb",
    ];
    pub fn from_path(input: &Path, extensions: &[&str]) -> Result<Self> {
        if !input.exists() {
            return Err(TraceForgeError::Io {
                path: input.display().to_string(),
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!(
                        "path does not exist: {}\n\
                         Pass a trace file or directory of trace files.\n\
                         Use `lix find` to discover traces automatically.",
                        input.display()
                    ),
                ),
            });
        }
        let mut files = Vec::new();
        if input.is_file() {
            if has_extension(input, extensions) {
                files.push(input.to_path_buf());
            }
        } else if input.is_dir() {
            for entry in ignore::WalkBuilder::new(input)
                .hidden(false)
                .git_ignore(true)
                .git_exclude(true)
                .ignore(true)
                .build()
            {
                let entry = entry.map_err(|e| TraceForgeError::Io {
                    path: input.display().to_string(),
                    source: std::io::Error::other(e.to_string()),
                })?;
                if entry.file_type().is_some_and(|t| t.is_file())
                    && has_extension(entry.path(), extensions)
                {
                    files.push(entry.path().to_path_buf());
                }
            }
        }
        files.sort();
        Ok(Self { files })
    }

    pub fn scan_common() -> Vec<PathBuf> {
        let mut found = Vec::new();
        let home = match std::env::var("HOME") {
            Ok(h) => h,
            Err(_) => return found,
        };
        for dir in Self::COMMON_TRACE_DIRS {
            let p = Path::new(&home).join(dir);
            if p.is_dir() {
                if let Ok(mut fs) = FileSet::from_path(&p, &["jsonl", "json"]) {
                    found.append(&mut fs.files);
                }
            }
        }
        for file in Self::COMMON_TRACE_FILES {
            let p = Path::new(&home).join(file);
            if p.is_file() {
                found.push(p);
            }
        }
        // Scan Hermes state-snapshots for state.db files
        let hermes_snapshots = Path::new(&home).join(".hermes/state-snapshots");
        if hermes_snapshots.is_dir() {
            if let Ok(dir) = std::fs::read_dir(&hermes_snapshots) {
                for entry in dir.flatten() {
                    let p = entry.path().join("state.db");
                    if p.is_file() {
                        found.push(p);
                    }
                }
            }
        }
        found.sort();
        found.dedup();
        found
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

fn has_extension(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| extensions.contains(&ext))
        .unwrap_or(false)
}
