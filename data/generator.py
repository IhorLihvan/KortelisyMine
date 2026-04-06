import os
import json
import hashlib
import requests
from loguru import logger

ROOT_DIR = "files"

ignored_files = [
    "manifest.json"
]

optional_files_startwith = [
    "options.txt",
    "config/"
]

def sha512_file(path):
    with open(path, 'rb') as f:
        return hashlib.sha512(f.read()).hexdigest()
    
file_list = []

for root, dirs, files in os.walk(ROOT_DIR):
    for file in files:
        full_paht = os.path.join(root, file)
        opt = False

        rel_path = os.path.relpath(full_paht, ROOT_DIR).replace("\\", "/")

        if rel_path in ignored_files: continue

        entry = {
            "name": file,
            "path": rel_path,
            "sha512": sha512_file(full_paht),
            "type": "temp"
        }

        if rel_path.startswith("mods/"):
            entry['type'] = "modification"

        for i in optional_files_startwith:
            if rel_path.startswith(i):
                entry["optional"] = True
                opt = True

        logger.info(f"Pushed '{entry["name"]}' file to manifest.json with {opt} optional parametr")

        file_list.append(entry)

with open("slug.json", 'r') as file:
    mods = json.load(file)
    
for slug in mods:
    url = f"https://api.modrinth.com/v2/project/{slug}/version?loaders=[%22forge%22]&game_versions=[%221.20.1%22]"
    response = requests.get(url=url).json()

    if response:
        latest = response[0]
        file_info = latest['files'][0]
        mod_entry = {
            "name": slug,   
            "url": file_info['url'],
            "sha512": file_info['hashes']['sha512'],
            "path": f"mods/{file_info['filename']}",
            "type": "modification"
        }
        file_list.append(mod_entry)
        logger.info(f"Pushed '{mod_entry["name"]}' modification to manifest.json")


manifest = {
    "version": "1.0.0",
    "files": file_list
}

with open(os.path.join(ROOT_DIR, "manifest.json"), 'w') as f:
    json.dump(manifest, f, indent=4)

logger.success("manifest.json successfuly generated")
input()