use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::error::{Result, TraceForgeError};

pub struct JsonlSink {
    writer: BufWriter<File>,
    path: String,
    rows_written: usize,
    bytes_written: usize,
}

impl JsonlSink {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| TraceForgeError::Io {
                path: parent.display().to_string(),
                source: e,
            })?;
        }
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .map_err(|e| TraceForgeError::Io {
                path: path.display().to_string(),
                source: e,
            })?;
        Ok(Self {
            writer: BufWriter::with_capacity(64 * 1024, file),
            path: path.display().to_string(),
            rows_written: 0,
            bytes_written: 0,
        })
    }

    pub fn write_row(&mut self, row: &[u8]) -> Result<()> {
        self.writer.write_all(row).map_err(|e| self.io_error(e))?;
        self.writer.write_all(b"\n").map_err(|e| self.io_error(e))?;
        self.rows_written += 1;
        self.bytes_written += row.len() + 1;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush().map_err(|e| self.io_error(e))
    }

    pub fn rows_written(&self) -> usize {
        self.rows_written
    }
    pub fn bytes_written(&self) -> usize {
        self.bytes_written
    }

    fn io_error(&self, source: std::io::Error) -> TraceForgeError {
        TraceForgeError::Io {
            path: self.path.clone(),
            source,
        }
    }
}
