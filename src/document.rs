use sha2::{Digest, Sha256};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Document {
    pub path: Option<PathBuf>,
    pub dirty: bool,
    pub last_saved_hash: Option<[u8; 32]>,
    pub last_modified: Option<std::time::SystemTime>,
}

impl Document {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self {
            path,
            dirty: false,
            last_saved_hash: None,
            last_modified: None,
        }
    }

    pub fn compute_hash(content: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}
