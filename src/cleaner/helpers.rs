pub fn dummy_hex(input: &str) -> String {
    let hash = blake3::hash(input.as_bytes());
    let mut s = String::with_capacity(8);
    for b in &hash.as_bytes()[0..4] {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

pub fn dummy_username(real: &str) -> String {
    format!("user_{}", dummy_hex(real))
}
