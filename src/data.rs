use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Deserialize)]
pub struct Version {
    pub version: String,
    pub manifest_sha512: String
}

#[derive(Deserialize)]
pub struct Manifest {
    pub version: String,
    pub files: Vec<FileEntry>
}

#[derive(Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub url: Option<String>,
    pub sha512: String,
    pub r#type: String,
    pub optional: Option<bool>
}

#[derive(Serialize, Deserialize, Default)]
pub struct IgnoredFiles {
    ignored_mods: Vec<String>
}

impl IgnoredFiles {
    pub fn save(&self) {
        let data = serde_json::to_string_pretty(self).unwrap();
        fs::write("ignored_files.json", data).ok();
    } 
    pub fn load() -> Self {
        fs::read_to_string("ignored_files.json")
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_default()
    }

    pub fn put(&mut self, path: String) {
        self.ignored_mods.push(path);
    }

    pub fn is_ignored(&self, path: &str) -> bool {
        self.ignored_mods.contains(&path.to_string())
    }
}