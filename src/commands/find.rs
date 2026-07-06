use crate::FileSet;

pub fn handle_find(_all: bool) -> Result<(), Box<dyn std::error::Error>> {
    let traces = FileSet::scan_common();
    if traces.is_empty() {
        println!("No trace files found in common locations.");
        println!();
        println!("Directories checked:");
        for d in FileSet::COMMON_TRACE_DIRS {
            println!("  ~/{d}");
        }
        println!("Files checked:");
        for f in FileSet::COMMON_TRACE_FILES {
            println!("  ~/{f}");
        }
        println!();
        println!("Pass a trace file directly:");
        println!("  lix extract ~/.claude/transcripts/session-123.jsonl");
    } else {
        println!("Found {} trace file(s):", traces.len());
        println!();
        for t in &traces {
            println!("  {}", t.display());
        }
        println!();
        println!("Extract them with:");
        println!("  lix extract <path-to-directory>");
    }
    Ok(())
}
