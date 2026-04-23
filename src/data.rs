use serde::Deserialize;

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