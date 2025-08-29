#!/usr/bin/env python3
"""
This script scans the SourcePack folder, generates/updates `elements_layering.json`,
and prints changes (green for additions, red for deletions). Subfolder paths are preserved.
"""

import json
from pathlib import Path

# ANSI escape codes
RED = "\033[91m"
GREEN = "\033[92m"
RESET = "\033[0m"

PACK_ROOT = Path("./SourcePack")
OUTPUT_JSON = Path("elements_layering.json")

# --- Safe load old JSON -----------------------------------------------------
if OUTPUT_JSON.exists():
    try:
        with open(OUTPUT_JSON, "r", encoding="utf-8") as f:
            content = f.read().strip()
            old_json = json.loads(content) if content else {}
    except json.JSONDecodeError:
        print(f"Warning: {OUTPUT_JSON} is invalid JSON, starting fresh.")
        old_json = {}
else:
    old_json = {}

# --- Helpers ----------------------------------------------------------------
def strip_png(name: str) -> str:
    if isinstance(name, str) and name.lower().endswith(".png"):
        return name[:-4]
    return name

def strip_png_recursive(obj):
    if isinstance(obj, dict):
        return {strip_png(k): strip_png_recursive(v) for k, v in obj.items()}
    elif isinstance(obj, list):
        return [strip_png_recursive(v) for v in obj]
    elif isinstance(obj, str):
        return strip_png(obj)
    else:
        return obj

def flatten(obj, prefix=""):
    """Flatten dicts into 'A/B/C': value form."""
    flat = {}
    if isinstance(obj, dict):
        for k, v in obj.items():
            k2 = strip_png(k)
            new_key = f"{prefix}/{k2}" if prefix else k2
            if isinstance(v, dict):
                flat.update(flatten(v, new_key))
            else:
                flat[new_key] = strip_png_recursive(v)
    return flat

def unflatten(flat):
    """Turn 'A/B/C': value back into nested dicts by top-level category."""
    nested = {}
    for full_key, val in flat.items():
        parts = full_key.split("/", 1)
        top = parts[0]
        rest = parts[1] if len(parts) > 1 else None
        if top not in nested:
            nested[top] = {}
        if rest:
            nested[top][rest] = val
        else:
            nested[top] = val
    return nested

# --- Build new flat mapping -------------------------------------------------
new_flat = {}
for folder in PACK_ROOT.iterdir():
    if not folder.is_dir():
        continue
    key = folder.name
    for file_path in sorted(folder.rglob("*")):
        if file_path.is_file():
            rel_path = str(file_path.relative_to(PACK_ROOT / key))
            rel_path = strip_png(rel_path)
            full_key = f"{key}/{rel_path}"
            new_flat[full_key] = [""]

# --- Flatten old JSON for comparison ----------------------------------------
old_json = strip_png_recursive(old_json)
old_flat = flatten(old_json)

# --- Merge & report ---------------------------------------------------------
merged_flat = {}
for k in sorted(new_flat.keys()):
    if k in old_flat:
        merged_flat[k] = old_flat[k]       # keep old value
    else:
        print(f"{GREEN}Added:   {k}{RESET}")
        merged_flat[k] = [""]              # new key

for k in old_flat.keys():
    if k not in new_flat:
        print(f"{RED}Deleted: {k}{RESET}")

# Turn flat back into nested by top-level category
merged_json = unflatten(merged_flat)

# --- Custom dumper with grouping --------------------------------------------
def dump_json_grouped(obj, indent=2, level=0, top_level=False):
    """
    - Keeps top-level categories intact.
    - Adds blank lines between top-level categories.
    - Inside each category: adds blank lines when the first subfolder changes.
    - Sorts keys case-insensitively (based on lowercase).
    """
    spaces = ' ' * (indent * level)
    if isinstance(obj, dict):
        # Sort keys alphabetically ignoring case
        keys = sorted(obj.keys(), key=lambda x: x.lower())
        lines = []
        for i, k in enumerate(keys):
            v = obj[k]
            comma = ',' if i < len(keys) - 1 else ''

            if top_level:
                if i > 0:
                    lines.append("")  # blank line between top-level categories
                lines.append(f'{spaces}  "{k}": {dump_json_grouped(v, indent, level+1)}{comma}')
            else:
                if i > 0:
                    prev_parent = keys[i-1].split("/", 1)[0]
                    curr_parent = k.split("/", 1)[0]
                    if prev_parent != curr_parent:
                        lines.append("")
                lines.append(f'{spaces}  "{k}": {dump_json_grouped(v, indent, level+1)}{comma}')

        return "{\n" + "\n".join(lines) + f"\n{spaces}}}"
    elif isinstance(obj, list):
        return "[" + ",".join(json.dumps(x) for x in obj) + "]"
    else:
        return json.dumps(obj)


# --- Write output -----------------------------------------------------------
with open(OUTPUT_JSON, "w", encoding="utf-8") as f:
    f.write(dump_json_grouped(merged_json, top_level=True) + "\n")

print(f"JSON file '{OUTPUT_JSON}' updated.")
