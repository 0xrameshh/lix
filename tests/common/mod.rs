use std::path::PathBuf;

pub fn fixture(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.join("tests").join("fixtures").join(name)
}
