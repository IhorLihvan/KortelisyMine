use std::{collections::HashMap, time::UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use std::fs;
use sha2::{Digest, Sha512};

pub fn hash_file_sha512(path: &str) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let mut hasher = Sha512::new();
    std::io::copy(&mut file, &mut hasher).ok()?;
    Some(format!("{:x}", hasher.finalize()))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CacheEntry {
    pub sha512: String,
    pub last_modified: u64,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Default)]
pub struct FileCache {
    pub files: HashMap<String, CacheEntry>,
}

impl FileCache {
    pub fn get_hash(&mut self, path: &str) -> String {
        let metadata = fs::metadata(path).expect("Failed to get metadata");
        let size = metadata.len();
        let modified = metadata.modified().unwrap()
            .duration_since(UNIX_EPOCH).unwrap().as_secs();
        

        if let Some(entry) = self.files.get(path) {
            if entry.size == size && entry.last_modified == modified {
                return entry.sha512.clone();
            }
        }
        
        let new_hash = hash_file_sha512(path).expect("Hash failed");

        self.files.insert(path.to_string(), CacheEntry {
            sha512: new_hash.clone(),
            last_modified: modified,
            size,
        });

        new_hash
    }

    pub fn load() -> Self {
        fs::read_to_string("cache.json")
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let data = serde_json::to_string_pretty(self).unwrap();
        fs::write("cache.json", data).ok();
    }
}