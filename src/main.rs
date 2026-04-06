use std::{fs, io::Write, path::Path};

use serde::Deserialize;
use sha2::{Sha256, Digest};

mod input;

fn hash_file(path: &str) -> Option<String> {
    let data = fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(data);
    Some(format!("{:x}", hasher.finalize()))
}

fn download_file(url: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Downloading {} on {}", path, url);

    let respone = reqwest::blocking::get(url)?;
    let bytes = respone.bytes()?;

    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(path)?;
    file.write_all(&bytes)?;

    Ok(())
}

#[derive(Deserialize)]
struct Manifest {
    version: String,
    files: Vec<FileEntry>
}

#[derive(Deserialize)]
struct FileEntry {
    path: String,
    url: String,
    sha256: String,
    optional: Option<bool>
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url_path_ip = input::input_string("Ip to file server:");
    let manifest_url = format!("http://{}:8000/manifest.json", url_path_ip);

    let mut update_count = 0;
    let mut delete_count = 0;

    println!("Url To Manifest: {}", &manifest_url);

    println!("Fetching manifest...");
    let manifest: Manifest = reqwest::blocking::get(&manifest_url).unwrap().json().unwrap();
    println!("Version: {}", manifest.version);

    let mut mods: Vec<String> = Vec::new();

    for file in manifest.files {
        let path = &file.path;
        let rel_path = Path::new(path);

        let needs_download = if !rel_path.exists() {
            true
        } else {
            
            match hash_file(path) {
                Some(local_hash) => local_hash != file.sha256 && !file.optional.unwrap(),
                None => true
            }
        };
        if needs_download {
            download_file(format!("http://{}:8000/{}", url_path_ip, file.url).as_str(), path)?;
            update_count += 1;
        } else {
            println!("\x1b[32m Ok {} \x1b[37m", path);
        }

        if rel_path.starts_with("mods") {
            mods.push(path.to_string());
        }
    }
    println!();

    if let Ok(entries) = fs::read_dir("mods")  {
        for entry in entries {
            if let Ok(entry) = entry {
                let ent = entry.path();
                let path = ent.to_str().unwrap();
                let mut index_found: Option<usize> = None;

                for (i, pa) in mods.iter().enumerate() {
                    if Path::new(pa) == path {
                        index_found = Some(i);
                        break;
                    }
                }
                match index_found {
                    Some(el) => {mods.remove(el);}
                    None => {
                        let _ = fs::remove_file(path);
                        delete_count += 1;
                        println!("\x1b[33mFile {} in mods has been deleted!\x1b[37m", path);
                    }
                }
            }
        }
    }    

    println!("\n\x1b[32m\u{1F600} Done!\x1b[37m\n");
    if update_count > 0 || delete_count > 0 {
        println!("Installed {} files | Deleted files {}", update_count, delete_count);
    } else {
        println!("No updates available!")
    }
    input::input_string("");
    Ok(())
}
