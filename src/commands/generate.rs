use std::io::Write;
use std::path::Path;

pub fn handle_generate(out_dir: &Path, count: usize) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(out_dir)?;
    let providers = [
        "claude-code",
        "codex",
        "droid",
        "hermes",
        "cursor",
        "pi",
        "openclaw",
    ];
    for i in 0..count {
        let provider = providers[i % providers.len()];
        let file_name = format!("synthetic_{}_{}.jsonl", provider, i + 1);
        let path = out_dir.join(&file_name);
        let mut f = std::fs::File::create(&path)?;

        let user_msg = format!("Synthetic user request {}", i + 1);
        let event = serde_json::json!({
            "type": "user",
            "content": user_msg,
            "timestamp": format!("2025-01-{:02}T00:00:00Z", (i % 12) + 1),
        });
        writeln!(f, "{}", serde_json::to_string(&event)?)?;
        println!("Created: {}", path.display());
    }
    println!(
        "\nGenerated {count} synthetic trace(s) in {}",
        out_dir.display()
    );
    println!(
        "Try: lix extract {} --out training.jsonl",
        out_dir.display()
    );
    Ok(())
}
