#!/usr/bin/env python3
"""Create a ready-to-install Iris ZIP with shaders/ at the archive root."""
from __future__ import annotations

import argparse
import hashlib
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "shaderpacks" / "HyperRealism-Stylized-PBR"
DEFAULT_OUTPUT = ROOT / "shaderpacks" / "dist" / "HyperRealism-Gilded-Dusk-MC-26.3.zip"
REQUIRED = (
    "LICENSE",
    "README.md",
    "shaders/shaders.properties",
    "shaders/lib/common/settings.glsl",
    "shaders/world0/final.fsh",
)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args()
    output = args.output.resolve()
    if not PACK.is_dir():
        parser.error(f"Shader source directory not found: {PACK}")
    for name in REQUIRED:
        if not (PACK / name).is_file():
            parser.error(f"Required pack file is missing: {name}")
    if output.is_relative_to(PACK.resolve()):
        parser.error("Output must be outside the source folder")

    output.parent.mkdir(parents=True, exist_ok=True)
    files = sorted(path for path in PACK.rglob("*") if path.is_file())
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for path in files:
            archive.write(path, path.relative_to(PACK).as_posix())

    with zipfile.ZipFile(output) as archive:
        names = set(archive.namelist())
        bad = archive.testzip()
        if bad is not None:
            parser.error(f"ZIP CRC check failed at {bad}")
        missing = [name for name in REQUIRED if name not in names]
        if missing:
            parser.error("ZIP is missing required files: " + ", ".join(missing))
        if any(name.startswith("HyperRealism-Stylized-PBR/") for name in names):
            parser.error("ZIP has an extra enclosing folder; Iris needs shaders/ at its root")
        shader_files = sum(name.startswith("shaders/") and not name.endswith("/") for name in names)
        if shader_files < 300:
            parser.error(f"ZIP appears incomplete: only {shader_files} shader files")

    digest = hashlib.sha256(output.read_bytes()).hexdigest()
    print(f"Built {output}")
    print(f"{shader_files} shader files; {output.stat().st_size:,} bytes; SHA-256 {digest}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
