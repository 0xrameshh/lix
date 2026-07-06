use crate::report::{FileReport, FileStatus};

#[derive(Default)]
pub struct FileAggregate {
    pub trace_type: String,
    pub ok: usize,
    pub dropped: usize,
    pub errors: usize,
    pub messages: usize,
    pub tool_calls: usize,
    pub replacements: usize,
}

impl FileAggregate {
    pub fn record(&mut self, fr: &FileReport) {
        if self.trace_type.is_empty() || matches!(fr.status, FileStatus::Ok) {
            self.trace_type = fr.trace_type.clone();
        }
        match fr.status {
            FileStatus::Ok => self.ok += 1,
            FileStatus::Error => self.errors += 1,
            _ => self.dropped += 1,
        }
        self.messages += fr.messages;
        self.tool_calls += fr.tool_calls;
        self.replacements += fr.clean.total_replacements;
    }

    pub fn final_status(&self) -> FileStatus {
        if self.ok > 0 {
            FileStatus::Ok
        } else if self.errors > 0 {
            FileStatus::Error
        } else {
            FileStatus::Empty
        }
    }
}
