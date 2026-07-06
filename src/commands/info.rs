use std::path::PathBuf;

pub fn handle_info(file: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let events = crate::read_all_events(&file)?;
    let mut type_counts: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    for ev in &events {
        let t = ev.r#type.as_deref().unwrap_or("unknown").to_string();
        *type_counts.entry(t).or_insert(0) += 1;
    }
    let tt = crate::detect(&events);
    let total = events.len();
    println!("File: {}", file.display());
    println!("Detected type: {:?}", tt);
    println!("Total events: {total}");
    for (t, c) in &type_counts {
        println!("  {t}: {c}");
    }
    Ok(())
}
