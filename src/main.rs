use std::{fs, io::Write, path::Path};
use colored::Colorize;
use indicatif::ProgressBar;
use std::collections::HashSet;

mod data;
mod cli;
mod hashsum;

use hashsum::{FileCache, hash_file_sha512};
use data::*;

const RAW_URL: &'static str = "https://raw.githubusercontent.com/IhorLihvan/KortelisyMine/data/data/files/";
const SUPPORT_MAJOR_VERSION: u16 = 1;

fn check_valid_version(version: &String) -> bool {
    let mut iterator = version.split(".");
    if let Some(el) = iterator.next() {
        if el.parse::<u16>().unwrap() > SUPPORT_MAJOR_VERSION {
            return false;
        }
    }
    true
}



fn download_file(url: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let respone = reqwest::blocking::get(url)?;
    let bytes = respone.bytes()?;

    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(path)?;
    file.write_all(&bytes)?;

    Ok(())
}

fn check_for_updates(version_data: String) -> bool {
    let remote_version: Version = serde_json::from_str(&version_data).unwrap();

    if !check_valid_version(&remote_version.version) {
        cli::stop_work_and_output(&format!("{}", "New Version of Manifest, update your Client Minecraft updater".yellow()));
    }

    if Path::new("manifest.json").exists() {
        let hash_local = hash_file_sha512("manifest.json").unwrap();

        if remote_version.manifest_sha512 == hash_local {
            return false;
        }
    }
    true
}

fn save_manifest(manifest_data: Vec<u8>) {
    fs::write("manifest.json", manifest_data).unwrap_or_else(|_| println!("Don`t saved version"));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let version_url: String = format!("{}version.json", RAW_URL);
    let manifest_url = format!("{}manifest.json", RAW_URL);

    println!("Checking for updates.");

    let version_remote_text = match reqwest::blocking::get(&version_url) {
        Ok(flx) => flx.text().unwrap(),
        Err(e) => {
            cli::stop_work_and_output(&format!("{}\n{}", "Unxpected Error, maybe check your internet connection".red(), e.to_string()));
            panic!("Process stopped");
        }
    };

    if !check_for_updates(version_remote_text) {
        println!("{}", "Updates not found".green());
        if !cli::confim_input("Continue installing") {
            cli::stop_work_and_output("");
        }
    }

    let mut update_count = 0;
    let mut delete_count = 0;
    let mut files_to_update: Vec<FileEntry> = Vec::new();
    let mut mods: Vec<String> = Vec::new();
    let mut file_to_delete: Vec<String> = Vec::new();
    let mut ignored_files: IgnoredFiles = IgnoredFiles::load();
    let mut file_cache = FileCache::load();

    println!("Fetching updates...");
    let manifest_file  = reqwest::blocking::get(&manifest_url).unwrap().bytes().unwrap().to_vec();
    let manifest: Manifest = serde_json::from_slice(&manifest_file).unwrap();
    println!("Received successfully.");

    for file in manifest.files {
        let path = &file.path;
        let rel_path = Path::new(path);

        let needs_download = if !rel_path.exists() {
            true
        } else {
            match file_cache.get_hash(path) {
                local_hash => local_hash != file.sha512 && !file.optional.unwrap_or(false),
            }
        };

        if &file.r#type == "modification" {
            mods.push(path.to_string());
        }

        if needs_download {
            println!("{}{}", "To install ".green(), &file.name);
            files_to_update.push(file);
        } else {
            println!("{} {}", "Ok".yellow(), &file.name);
        }

        
    }
    

    if files_to_update.len() > 0 {
        if cli::confim_input(&format!("Confirm update {} files", files_to_update.len())) {
            let steps = files_to_update.len();
            let pb = ProgressBar::new(steps as u64).with_message("down:");

            for file in files_to_update {
                match &file.url {
                    Some(el) => download_file(el, &file.path)?,
                    None => download_file(format!("{}{}", RAW_URL, &file.path).as_str(), &file.path)?
                };
                file_cache.get_hash(&file.path);
                update_count+=1;
                pb.inc(1);
            }
            pb.finish_with_message("Done!");
            println!("{} {} {}", "Successfully Installed ".green(), update_count, " files".green());
        }
    }

    file_cache.save();
    save_manifest(manifest_file);
    
    let mods_set: HashSet<_> = mods.into_iter().collect();

    if let Ok(entries) = fs::read_dir("mods") {
        for entry in entries.flatten() {
            let path_buf = entry.path();
            
            let path_str = path_buf.to_str().unwrap().replace("\\", "/");

            if ignored_files.is_ignored(&path_str) { continue; }

            if !mods_set.contains(&path_str) {
                file_to_delete.push(path_str);
            }
        }
    }

    if file_to_delete.len() > 0 {
        println!("Found {} files in mods folder, please make sure you save them to contiune", file_to_delete.len());

        if cli::confim_input("Start") {
            for path in file_to_delete {
                    let conf = cli::input_string(&format!("Delete {} (y, ignore):", path));
                    if conf == "y" {
                        let _ = fs::remove_file(path.as_str());
                        delete_count+=1;
                        println!("File {} in mods has been deleted!", path);
                    } else if conf == "ignore" {
                        ignored_files.put(path);
                    }
                
            }
            ignored_files.save();
        }
    }
    
    if update_count > 0 || delete_count > 0 {
        println!("Installed {} files | Deleted files {}", update_count, delete_count);
    }

    println!("{}", "Done!".bold().green());

    cli::input();
    Ok(())
}
