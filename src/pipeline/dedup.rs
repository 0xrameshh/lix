use std::collections::HashSet;
use std::sync::Mutex;

pub struct DedupTracker {
    seen: Mutex<HashSet<String>>,
}

impl DedupTracker {
    pub fn new() -> Self {
        Self {
            seen: Mutex::new(HashSet::new()),
        }
    }

    pub fn try_insert(&self, session_id: &str) -> bool {
        let mut seen = self.seen.lock().expect("dedup lock");
        seen.insert(session_id.to_string())
    }
}

impl Default for DedupTracker {
    fn default() -> Self {
        Self::new()
    }
}
