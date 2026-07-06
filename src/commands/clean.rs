use std::path::PathBuf;

use crate::{Cleaner, JsonlSink, LineReader};

pub fn handle_clean(input: PathBuf, out: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let cleaner = Cleaner::new();
    let mut sink = JsonlSink::open(&out)?;
    let reader = LineReader::open(&input)?;
    let mut total = 0usize;
    for line in reader {
        let ev = line?;
        let json_str = serde_json::to_string(&ev.raw).unwrap_or_default();
        let mut text = json_str;
        cleaner.clean_text(&mut text, &mut Default::default());
        sink.write_row(text.as_bytes())?;
        total += 1;
    }
    sink.flush()?;
    eprintln!(
        "Cleaned {total} lines from {} -> {}",
        input.display(),
        out.display()
    );
    Ok(())
}
