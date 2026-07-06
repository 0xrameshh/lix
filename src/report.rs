use serde::{Deserialize, Serialize};

use crate::cleaner::CleanReport;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FileReport {
    pub source: String,
    pub trace_type: String,
    pub status: FileStatus,
    pub messages: usize,
    pub tool_calls: usize,
    pub dropped_reason: Option<String>,
    #[serde(default)]
    pub clean: CleanReport,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileStatus {
    #[default]
    Ok,
    DroppedIncomplete,
    DroppedSynthetic,
    DroppedModelFilter,
    DroppedDuplicate,
    Empty,
    Unsupported,
    Error,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ExtractionReport {
    pub files_total: usize,
    pub files_ok: usize,
    pub files_dropped: usize,
    pub files_errored: usize,
    pub rows_written: usize,
    pub total_replacements: usize,
    pub elapsed_secs: f64,
    #[serde(skip)]
    pub files: Vec<FileReport>,
}

impl ExtractionReport {
    pub fn record_file(&mut self, fr: FileReport) {
        self.files_total += 1;
        match fr.status {
            FileStatus::Ok => self.files_ok += 1,
            FileStatus::DroppedIncomplete
            | FileStatus::DroppedSynthetic
            | FileStatus::DroppedModelFilter
            | FileStatus::DroppedDuplicate
            | FileStatus::Empty => self.files_dropped += 1,
            FileStatus::Unsupported => self.files_dropped += 1,
            FileStatus::Error => self.files_errored += 1,
        }
        self.total_replacements += fr.clean.total_replacements;
        self.files.push(fr);
    }
}
