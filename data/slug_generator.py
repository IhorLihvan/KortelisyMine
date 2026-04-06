import hashlib
import requests
import json
import os
import shutil

mods_path = input("mods path:")
slugs = []

os.makedirs("files/mods", exist_ok=True)

Root, _, files = next(os.walk(mods_path))

for file in files:
    try:
        with open(os.path.join(Root, file), 'rb') as f:
            hash = hashlib.sha1(f.read()).hexdigest()
        
        project_id = requests.get(f"https://api.modrinth.com/v2/version_file/{hash}").json()["project_id"]

        slug = requests.get(f"https://api.modrinth.com/v2/project/{project_id}").json()["slug"]
        slugs.append(slug)
    except Exception as e:
        print("Error on", file)
        shutil.copy(os.path.join(Root, file), "files/mods/")


with open("slug.json", 'w') as file:
    json.dump(slugs, file, indent=4)

print("successfuly generated slug.json")
input()