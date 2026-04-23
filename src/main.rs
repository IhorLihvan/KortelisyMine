use std::{fs, io::Write, path::Path};
use colored::Colorize;
use reqwest::blocking::Response;
use std::thread;
use std::time::Duration;
use sha2::{Digest, Sha512};
use indicatif::ProgressBar;

mod data;
mod cli;

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

fn hash_file_sha512(path: &str) -> Option<String> {
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

fn check_for_updates(version_data: Response) -> bool {
    let remote_version: Version = version_data.json::<Version>().unwrap();

    if !check_valid_version(&remote_version.version) {
        cli::stop_work_and_output(&format!("{}", "New Version of Manifest, update your Client Minecraft updater".yellow()));
    }

    if fs::exists("manifest.json").unwrap() {
        let hash_local = hash_file_sha512("manifest.json").unwrap();

        if remote_version.manifest_sha512 == hash_local {
            return false;
        }
    }
    true
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let version_url: String = format!("{}version.json", RAW_URL);
    let manifest_url = format!("{}manifest.json", RAW_URL);
    println!("Checking for updates.");

    let version_remote_data = match reqwest::blocking::get(&version_url) {
        Ok(flx) => flx,
        Err(e) => {
            cli::stop_work_and_output(&format!("{}\n{}", "Unxpected Error, maybe check your internet connection".red(), e.to_string()));
            panic!("Process stopped");
        }
    };

    version_remote_data.text();

    if !check_for_updates(version_remote_data) {
        println!("{}", "Updates not found".green());
        if !cli::confim_input("Continue installing") {
            cli::stop_work_and_output("");
        }
    }

    let mut update_count = 0;
    let mut delete_count = 0;
    let mut files_to_update: Vec<FileEntry> = Vec::new();
    let mut mods: Vec<String> = Vec::new();

    println!("Fetching updates...");
    let manifest: Manifest = reqwest::blocking::get(&manifest_url).unwrap().json().unwrap();
    println!("Received successfully.");

    for file in manifest.files {
        let path = &file.path;
        let rel_path = Path::new(path);

        let needs_download = if !rel_path.exists() {
            true
        } else {
            match hash_file_sha512(path) {
                Some(local_hash) => local_hash != file.sha512 && !file.optional.unwrap_or(false),
                None => true
            }
        };

        if &file.r#type == "modification" {
            mods.push(path.to_string());
        }

        if needs_download {
            println!("{} {}", "found to install".green(), &file.name);
            files_to_update.push(file);
        } else {
            println!("{} {}", "Ok".yellow(), &file.name);
        }

        
    }

    if files_to_update.len()>0 {
        if cli::confim_input(&format!("Confirm update {} files", files_to_update.len())) {
            let steps = files_to_update.len();
            let pb = ProgressBar::new(steps as u64);

            for file in files_to_update {
                match &file.url {
                    Some(el) => download_file(el, &file.path)?,
                    None => download_file(format!("{}{}", RAW_URL, &file.path).as_str(), &file.path)?
                };
                update_count+=1;
                pb.inc(1);
            }
            pb.finish_with_message("Done!");
            println!("{} {} {}", "Successfully Installed ".green(), update_count, " files".green());
        }
    }

    // println!();
    // let mut file_to_delete: Vec<String> = Vec::new();

    // if let Ok(entries) = fs::read_dir("mods")  {
    //     for entry in entries {
    //         if let Ok(entry) = entry {
    //             let ent = entry.path();
    //             let path = ent.to_str().unwrap();
    //             let mut index_found: Option<usize> = None;

    //             for (i, pa) in mods.iter().enumerate() {
    //                 if Path::new(pa) == path {
    //                     index_found = Some(i);
    //                     break;
    //                 }
    //             }
    //             match index_found {
    //                 Some(el) => {mods.remove(el);}
    //                 None => {
    //                     file_to_delete.push(path.to_string());
    //                     delete_count += 1;
    //                 }
    //             }
    //         }
    //     }
    // }
    // if delete_count > 0 {
    //     println!("Found {} files in mods folder, please make sure you save them to contiune", delete_count);
    //     let first_conf = cli::input_string("Delete files(N, y):");
    //     if first_conf.to_lowercase() == "y" {
    //         let second_conf = cli::input_string("You sure(N, y):");

    //         if first_conf.to_lowercase() == second_conf.to_lowercase() && first_conf.to_lowercase() == "y" {
    //         for path in file_to_delete {
    //             let _ = fs::remove_file(path.as_str());
    //             println!("\x1b[33mFile {} in mods has been deleted!\x1b[37m", path);
    //         }
    //     }
    //     }
        

        
    // }

    // println!("\n\x1b[32m\u{1F600} Done!\x1b[37m\n");
    // if update_count > 0 || delete_count > 0 {
    //     println!("Installed {} files | Deleted files {}", update_count, delete_count);
    // } else {
    //     println!("No updates available!")
    // }

    cli::input();
    Ok(())
}
