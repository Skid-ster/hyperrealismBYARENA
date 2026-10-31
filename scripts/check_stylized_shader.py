#!/usr/bin/env python3
"""Structural and provenance checks for the optional experimental Iris shader."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "shaderpacks" / "HyperRealism-Stylized-PBR"
SHADERS = PACK / "shaders"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def main() -> int:
    settings = (SHADERS / "lib/common/settings.glsl").read_text(encoding="utf-8")
    processing = (SHADERS / "lib/post/processing.glsl").read_text(encoding="utf-8")
    tonemap = (SHADERS / "lib/post/tonemap.glsl").read_text(encoding="utf-8")
    properties = (SHADERS / "shaders.properties").read_text(encoding="utf-8")
    english = (SHADERS / "lang/en_US.lang").read_text(encoding="utf-8")
    material = (SHADERS / "lib/common/material.glsl").read_text(encoding="utf-8")
    readme = (PACK / "README.md").read_text(encoding="utf-8")
    license_text = (PACK / "LICENSE").read_text(encoding="utf-8")

    require("#define PBR_MODE 1" in settings, "LabPBR 1.3 must be enabled by default")
    require("#define STYLE_STRENGTH 70" in settings, "Expected stylized default is missing")
    require("STYLE_STRENGTH" in properties and "STYLE_STRENGTH" in english,
            "Style control must be visible and labeled in Iris settings")
    require("hyperrealismGrade" in processing and "coolShadow" in processing
            and "warmHighlight" in processing, "Teal/amber grade is incomplete")
    require("#define tonemap agxTonemap" in tonemap, "Expected AgX default is missing")
    require("materialFromSpecularMap" in material and "specularData" in material,
            "Upstream LabPBR material parsing is missing")
    require("6a591097b6348155a4ba5a1aab92b79a2c9896d3" in readme,
            "Upstream source revision must remain pinned in the shader notes")
    require("MIT License" in license_text and "Copyright (c) 2024 Josh Britain" in license_text,
            "Upstream MIT notice must be retained")
    require((PACK / "UPSTREAM_GLIMMER_README.md").is_file()
            and (SHADERS / "textures/LICENSE-MIT_tony_mc_mapface").is_file(),
            "Upstream notices and texture license must be retained")
    files = [path for path in SHADERS.rglob("*") if path.is_file()]
    require(len(files) >= 300, f"Shader source tree looks incomplete: {len(files)} files")
    print(f"Shader structure and MIT provenance checks passed ({len(files)} source/runtime files).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
