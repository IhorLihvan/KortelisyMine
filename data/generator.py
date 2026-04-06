import os
import json
import hashlib

ROOT_DIR = "server"

ignored_files = [
    "manifest.json",
    "updates-release.exe",
    "index.html",
    "favicon.ico"
]

def sha256_file(path):
    with open(path, 'rb') as f:
        return hashlib.sha256(f.read()).hexdigest()
    
file_list = []

for root, dirs, files in os.walk(ROOT_DIR):
    for file in files:
        full_paht = os.path.join(root, file)

        rel_path = os.path.relpath(full_paht, ROOT_DIR).replace("\\", "/")

        if rel_path in ignored_files: continue

        entry = {
            "path": rel_path,
            "sha256": sha256_file(full_paht),
            "url": rel_path
        }

        if rel_path.startswith("config/") or rel_path.startswith("options"):
            entry["optional"] = True

        file_list.append(entry)

manifest = {
    "version": "1.0.0",
    "files": file_list
}

with open(os.path.join(ROOT_DIR, "manifest.json"), 'w') as f:
    json.dump(manifest, f, indent=4)

print("manifest.json generated")
input()