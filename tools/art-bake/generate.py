"""Generates each tiling texture in a recipe through a local ComfyUI, skipping finished ones.

Usage: generate.py <recipe.json> <output folder> <ComfyUI output folder>
"""
import json
import shutil
import sys
import time
import urllib.request
from pathlib import Path

SERVER = "http://127.0.0.1:8188"


def post(path, body):
    request = urllib.request.Request(
        SERVER + path,
        data=json.dumps(body).encode(),
        headers={"Content-Type": "application/json"},
    )
    return json.load(urllib.request.urlopen(request))


def get(path):
    return json.load(urllib.request.urlopen(SERVER + path))


def workflow(recipe, texture, prefix):
    return {
        "1": {"class_type": "CheckpointLoaderSimple", "inputs": {"ckpt_name": recipe["model"]}},
        "2": {
            "class_type": "SeamlessTile",
            "inputs": {"model": ["1", 0], "tiling": "enable", "copy_model": "Make a copy"},
        },
        "3": {
            "class_type": "CLIPTextEncode",
            "inputs": {"text": texture["prompt"], "clip": ["1", 1]},
        },
        "4": {
            "class_type": "CLIPTextEncode",
            "inputs": {"text": recipe["negative"], "clip": ["1", 1]},
        },
        "5": {
            "class_type": "EmptyLatentImage",
            "inputs": {"width": 1024, "height": 1024, "batch_size": 1},
        },
        "6": {
            "class_type": "KSampler",
            "inputs": {
                "model": ["2", 0],
                "positive": ["3", 0],
                "negative": ["4", 0],
                "latent_image": ["5", 0],
                "seed": texture["seed"],
                "steps": texture["steps"],
                "cfg": texture["cfg"],
                "sampler_name": "dpmpp_2m",
                "scheduler": "karras",
                "denoise": 1.0,
            },
        },
        "7": {
            "class_type": "CircularVAEDecode",
            "inputs": {"samples": ["6", 0], "vae": ["1", 2], "tiling": "enable"},
        },
        "8": {
            "class_type": "SaveImage",
            "inputs": {"images": ["7", 0], "filename_prefix": prefix},
        },
    }


def generate(recipe, texture, prefix):
    prompt = workflow(recipe, texture, prefix)
    prompt_id = post("/prompt", {"prompt": prompt})["prompt_id"]
    while True:
        entry = get("/history/" + prompt_id).get(prompt_id)
        if entry and entry.get("status", {}).get("completed"):
            for output in entry["outputs"].values():
                return output["images"][0]["filename"]
        if entry and entry.get("status", {}).get("status_str") == "error":
            raise RuntimeError(json.dumps(entry["status"])[:800])
        time.sleep(0.5)


def main():
    recipe = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
    out = Path(sys.argv[2])
    comfy_output = Path(sys.argv[3])
    out.mkdir(parents=True, exist_ok=True)
    for name, texture in recipe["textures"].items():
        target = out / f"{name}.png"
        if target.exists():
            continue
        file = generate(recipe, texture, f"art-bake/{name}")
        shutil.copy(comfy_output / "art-bake" / file, target)
        print(target.name, flush=True)


main()
