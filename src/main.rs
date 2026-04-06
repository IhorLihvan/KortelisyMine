use std::{fs, io::Write, path::Path};

use serde::Deserialize;
use sha2::{Digest, Sha512};

mod input;

const RAW_URL: &'static str = "https://raw.githubusercontent.com/IhorLihvan/KortelisyMine/data/data/files/";

fn hash_file(path: &str) -> Option<String> {
    let data = fs::read(path).ok()?;
    let mut hasher = Sha512::new();
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
    url: Option<String>,
    sha512: String,
    r#type: String,
    optional: Option<bool>
}

fn check_valid_version(version: &String) -> bool {
    let mut iterator = version.split(".");
    if let Some(el) = iterator.next() {
        if el.parse::<usize>().unwrap() > 1 {
            return false;
        }
    }
    true
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_url = format!("{}manifest.json", RAW_URL);

    let mut update_count = 0;
    let mut delete_count = 0;

    println!("Url To Manifest: {}", &manifest_url);

    println!("Fetching manifest...");
    let manifest: Manifest = reqwest::blocking::get(&manifest_url).unwrap().json().unwrap();
    
    if check_valid_version(&manifest.version) {
        if input::input_string("Start Updating(y, N):").to_lowercase() != "y" {
            return Ok(())
        }

        let mut mods: Vec<String> = Vec::new();

        for file in manifest.files {
            let path = &file.path;
            let rel_path = Path::new(path);

            let needs_download = if !rel_path.exists() {
                true
            } else {
                
                match hash_file(path) {
                    Some(local_hash) => local_hash != file.sha512 && !file.optional.unwrap_or(false),
                    None => true
                }
            };
            if needs_download {
                match &file.url {
                    Some(el) => download_file(el, path)?,
                    None => download_file(format!("{}{}", RAW_URL, path).as_str(), path)?
                };
                update_count += 1;

            } else {
                println!("\x1b[32m Ok {} \x1b[37m", path);
            }

            if &file.r#type == "modification" {
                mods.push(path.to_string());
            }
        }
        println!();
        let mut file_to_delete: Vec<String> = Vec::new();

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
                            file_to_delete.push(path.to_string());
                            delete_count += 1;
                        }
                    }
                }
            }
        }
        if delete_count > 0 {
            println!("Found {} files in mods folder, please make sure you save them to contiune", delete_count);
            let first_conf = input::input_string("Delete files(N, y):");
            if first_conf.to_lowercase() == "y" {
                let second_conf = input::input_string("You sure(N, y):");

                if first_conf.to_lowercase() == second_conf.to_lowercase() && first_conf.to_lowercase() == "y" {
                for path in file_to_delete {
                    let _ = fs::remove_file(path.as_str());
                    println!("\x1b[33mFile {} in mods has been deleted!\x1b[37m", path);
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
    } else {
        println!("New Version of Manifest, update your Client Minecraft updater");
    }
    input::input_string("");
    Ok(())
}
