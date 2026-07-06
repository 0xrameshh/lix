use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CleanReport {
    pub api_keys: usize,
    pub home_paths: usize,
    pub emails: usize,
    pub jwt: usize,
    pub bearer: usize,
    pub generic_secret: usize,
    pub credential_url: usize,
    pub private_key_block: usize,
    pub env_assignment: usize,
    pub query_secret: usize,
    pub total_replacements: usize,
}

impl CleanReport {
    pub fn bump(&mut self, kind: &str) {
        self.total_replacements += 1;
        match kind {
            "api_key" => self.api_keys += 1,
            "home_path" => self.home_paths += 1,
            "email" => self.emails += 1,
            "jwt" => self.jwt += 1,
            "bearer" => self.bearer += 1,
            "generic_secret" => self.generic_secret += 1,
            "credential_url" => self.credential_url += 1,
            "private_key_block" => self.private_key_block += 1,
            "env_assignment" => self.env_assignment += 1,
            "query_secret" => self.query_secret += 1,
            _ => {}
        }
    }
}
