#!/usr/bin/env python3
"""Audit all blockstate -> model -> texture references in a pinned Minecraft asset mirror.

This reads JSON metadata only. It does not copy or emit Minecraft image assets.
"""
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from typing import Any

PINNED_REPO = "https://github.com/InventivetalentDev/minecraft-assets"
PINNED_COMMIT = "6c088630459d8ca369ef0d6c07636d7d63a3b0f1"


def read_json(path: Path) -> Any:
    with path.open(encoding="utf-8") as f:
        return json.load(f)


def resource_path(value: str) -> str:
    if ":" not in value:
        return "minecraft:" + value
    return value


def blockstate_models(data: dict[str, Any]) -> list[str]:
    found: list[str] = []

    def add_model(spec: Any) -> None:
        if isinstance(spec, dict) and isinstance(spec.get("model"), str):
            found.append(spec["model"])
        elif isinstance(spec, list):
            for part in spec:
                add_model(part)

    for spec in data.get("variants", {}).values():
        add_model(spec)
    for part in data.get("multipart", []):
        add_model(part.get("apply"))
    return found


def build_model_index(models_root: Path) -> dict[str, Path]:
    result = {}
    for p in models_root.rglob("*.json"):
        result["minecraft:" + p.relative_to(models_root).with_suffix("").as_posix()] = p
    return result


def model_textures(model_id: str, index: dict[str, Path], memo: dict[str, dict[str, str]],
                   problems: set[str], visiting: set[str] | None = None) -> dict[str, str]:
    model_id = resource_path(model_id)
    if model_id in memo:
        return memo[model_id]
    if visiting is None:
        visiting = set()
    if model_id in visiting:
        problems.add("model parent cycle: " + model_id)
        return {}
    path = index.get(model_id)
    if path is None:
        problems.add("missing model: " + model_id)
        return {}
    visiting.add(model_id)
    data = read_json(path)
    textures: dict[str, str] = {}
    parent = data.get("parent")
    if isinstance(parent, str):
        textures.update(model_textures(parent, index, memo, problems, visiting))
    own = data.get("textures", {})
    if isinstance(own, dict):
        for key, value in own.items():
            if isinstance(value, str):
                textures[key] = value
            elif isinstance(value, dict) and isinstance(value.get("sprite"), str):
                # Minecraft 26.3 metadata can wrap a sprite with rendering
                # hints such as force_translucent. The audit follows the sprite
                # path while leaving those model/render properties untouched.
                textures[key] = value["sprite"]
    visiting.remove(model_id)
    memo[model_id] = textures
    return textures


def expand_texture(key: str, textures: dict[str, str], problems: set[str], seen=None) -> str | None:
    if seen is None:
        seen = set()
    if key in seen:
        problems.add("texture variable cycle: " + " -> ".join((*seen, key)))
        return None
    seen.add(key)
    value = textures.get(key)
    if value is None:
        problems.add("unresolved texture variable: #" + key)
        return None
    if value.startswith("#"):
        return expand_texture(value[1:], textures, problems, seen)
    return resource_path(value)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--assets", type=Path, required=True,
                        help="checkout of the pinned asset-mirror commit")
    parser.add_argument("--manifest", type=Path, default=Path("work/material_manifest.json"))
    parser.add_argument("--output", type=Path, default=Path("catalog/vanilla_26.3_full_block_coverage.json"))
    args = parser.parse_args()

    try:
        source_commit = subprocess.check_output(
            ["git", "-C", str(args.assets), "rev-parse", "HEAD"],
            text=True,
            stderr=subprocess.DEVNULL,
        ).strip()
    except (OSError, subprocess.CalledProcessError):
        parser.error("--assets must be a Git checkout of the pinned metadata mirror")
    if source_commit != PINNED_COMMIT:
        parser.error(f"--assets must be pinned at {PINNED_COMMIT}, got {source_commit}")

    root = args.assets / "assets/minecraft"
    states_root = root / "blockstates"
    models_root = root / "models"
    if not states_root.is_dir() or not models_root.is_dir():
        parser.error("--assets must contain assets/minecraft/blockstates and models")

    manifest = read_json(args.manifest)
    covered = {item["name"] for item in manifest.get("textures", [])}
    model_index = build_model_index(models_root)
    model_memo: dict[str, dict[str, str]] = {}
    errors: set[str] = set()
    states: dict[str, dict[str, Any]] = {}

    for path in sorted(states_root.rglob("*.json")):
        state_id = path.relative_to(states_root).with_suffix("").as_posix()
        state_json = read_json(path)
        models = sorted(set(blockstate_models(state_json)))
        refs: set[str] = set()
        for model in models:
            for key in model_textures(model, model_index, model_memo, errors):
                resolved = expand_texture(key, model_textures(model, model_index, model_memo, errors), errors)
                if resolved:
                    refs.add(resolved)
        block_texture_names = sorted(ref.removeprefix("minecraft:block/")
                                     for ref in refs if ref.startswith("minecraft:block/"))
        other_textures = sorted(ref for ref in refs if not ref.startswith("minecraft:block/"))
        states[state_id] = {
            "models": models,
            "block_textures": block_texture_names,
            "other_textures": other_textures,
            "uncovered_block_textures": [name for name in block_texture_names if name not in covered],
        }

    all_block_textures = sorted({name for state in states.values() for name in state["block_textures"]})
    covered_refs = sorted(set(all_block_textures) & covered)
    uncovered_refs = sorted(set(all_block_textures) - covered)
    summary = {
        "blockstates": len(states),
        "block_model_files": sum(1 for p in models_root.rglob("*.json")
                                  if p.relative_to(models_root).parts[0] == "block"),
        "unique_block_texture_references": len(all_block_textures),
        "currently_covered_block_texture_references": len(covered_refs),
        "uncovered_block_texture_references": len(uncovered_refs),
        "block_texture_references_not_referenced_by_any_blockstate": sorted(covered - set(all_block_textures)),
        "states_with_uncovered_textures": sum(bool(s["uncovered_block_textures"]) for s in states.values()),
        "metadata_resolution_problems": sorted(errors),
    }
    result = {
        "version": 1,
        "source": PINNED_REPO,
        "source_commit": PINNED_COMMIT,
        "method": "Every blockstate JSON; resolve variant and multipart model IDs, recursively inherit model parents and texture variables. Texture names only; no Minecraft image data.",
        "summary": summary,
        "covered_block_textures": covered_refs,
        "uncovered_block_textures": uncovered_refs,
        "blockstates": states,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(summary, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
