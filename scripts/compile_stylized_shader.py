#!/usr/bin/env python3
"""Compile the Iris GLSL stages for the Gilded Dusk shader without launching MC.

This expands Iris-style #include paths, injects the public Iris 26.3 standard
macros, and compiles ordinary stages with glslangValidator. It also compiles a
second configuration with all the optional visual features enabled and
Distant Horizons macros present. ColorWheel stages are intentionally excluded:
they require ColorWheel's runtime shader injection.

This is a source/compiler check, not a replacement for launching the shader in
Minecraft or validating a particular GPU/driver.
"""
from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SHADERS = ROOT / "shaderpacks" / "HyperRealism-Stylized-PBR" / "shaders"

# Iris 26.3's StandardMacros and WorldRenderingPhase values. Update these
# together with the target matrix when the supported Minecraft/Iris release is
# deliberately changed. Source: github.com/IrisShaders/Iris/tree/26.3
RENDER_STAGES = (
    "NONE", "SKY", "SUNSET", "CUSTOM_SKY", "SUN", "MOON", "STARS", "VOID",
    "TERRAIN_SOLID", "TERRAIN_CUTOUT_MIPPED", "TERRAIN_CUTOUT", "ENTITIES",
    "BLOCK_ENTITIES", "DESTROY", "OUTLINE", "DEBUG", "HAND_SOLID",
    "TERRAIN_TRANSLUCENT", "TRIPWIRE", "PARTICLES", "CLOUDS", "RAIN_SNOW",
    "WORLD_BORDER", "HAND_TRANSLUCENT",
)
DH_BLOCKS = (
    "UNKNOWN", "LEAVES", "STONE", "WOOD", "METAL", "DIRT", "LAVA",
    "DEEPSLATE", "SNOW", "SAND", "TERRACOTTA", "NETHER_STONE", "WATER",
    "GRASS", "AIR", "ILLUMINATED",
)
IRIS_MACROS = """#define IS_IRIS
#define MC_VERSION 260300
#define IRIS_VERSION 11107
#define MC_GL_VERSION 46
#define MC_GLSL_VERSION 460
#define MC_HAND_DEPTH 0.125
#define MC_NORMAL_MAP
#define MC_SPECULAR_MAP
#define MC_TEXTURE_FORMAT_LAB_PBR
#define MC_TEXTURE_FORMAT_LAB_PBR_1_3
#define MC_RENDER_QUALITY 1.0
#define MC_SHADOW_QUALITY 1.0
#define MC_MIPMAP_LEVEL 4
#define MC_OS_LINUX
#define MC_GL_VENDOR_OTHER
#define MC_GL_RENDERER_OTHER
""" + "".join(
    f"#define MC_RENDER_STAGE_{name} {value}\n"
    for value, name in enumerate(RENDER_STAGES)
)
DH_MACROS = "#define DISTANT_HORIZONS\n" + "".join(
    f"#define DH_BLOCK_{name} {value}\n"
    for value, name in enumerate(DH_BLOCKS)
)

FEATURE_CHANGES = (
    ("// #define AUTO_EXPOSURE", "#define AUTO_EXPOSURE"),
    ("// #define FLOODFILL", "#define FLOODFILL"),
    ("// #define INFINITE_OCEAN", "#define INFINITE_OCEAN"),
    ("// #define WATER_PARALLAX", "#define WATER_PARALLAX"),
    ("// #define CAUSTICS", "#define CAUSTICS"),
    ("// #define MOTION_BLUR", "#define MOTION_BLUR"),
    ("// #define PARALLAX", "#define PARALLAX"),
    ("// #define SSR_JITTER", "#define SSR_JITTER"),
    ("#define STYLE_STRENGTH 70", "#define STYLE_STRENGTH 0"),
)


def expand_includes(root: Path, source: Path, chain: tuple[Path, ...] = ()) -> str:
    if source in chain:
        raise RuntimeError("Cyclic shader include: " + " -> ".join(map(str, chain + (source,))))
    expanded: list[str] = []
    for line in source.read_text(encoding="utf-8").splitlines():
        match = re.match(r'\s*#include\s+"([^"]+)"', line)
        if not match:
            expanded.append(line)
            continue
        include = match.group(1)
        target = root / include.lstrip("/") if include.startswith("/") else source.parent / include
        if not target.is_file():
            raise RuntimeError(f"Missing include {include!r} in {source}")
        expanded.append(expand_includes(root, target, chain + (source,)))
    return "\n".join(expanded)


def prepare_variant(destination: Path, changes: tuple[tuple[str, str], ...]) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copytree(SHADERS, destination)
    settings = destination / "lib/common/settings.glsl"
    text = settings.read_text(encoding="utf-8")
    for before, after in changes:
        if before not in text:
            raise RuntimeError(f"Expected settings line not found: {before}")
        text = text.replace(before, after, 1)
    settings.write_text(text, encoding="utf-8")


def compile_variant(
    label: str,
    root: Path,
    compiler: str,
    output_dir: Path,
    distant_horizons: bool,
) -> tuple[int, int, list[str]]:
    macros = IRIS_MACROS + (DH_MACROS if distant_horizons else "")
    stages = sorted(
        stage
        for world in root.iterdir()
        if world.is_dir() and world.name.startswith("world")
        for stage in (*world.glob("*.vsh"), *world.glob("*.fsh"), *world.glob("*.csh"))
        if not stage.name.startswith("clrwl_")
        and (distant_horizons or not stage.name.startswith("dh_"))
    )
    failures: list[str] = []
    for index, stage in enumerate(stages):
        source = expand_includes(root, stage)
        version, separator, remainder = source.partition("\n")
        if not separator or not version.startswith("#version"):
            failures.append(f"{stage.relative_to(root)}: expected #version as the first line")
            continue
        extra = ""
        if distant_horizons and stage.name.startswith("dh_") and stage.suffix == ".vsh":
            # Iris/Distant Horizons supplies this vertex attribute at runtime.
            extra = "in int dhMaterialId;\n"
        glsl = output_dir / f"{label}_{index}{'.vert' if stage.suffix == '.vsh' else '.frag' if stage.suffix == '.fsh' else '.comp'}"
        glsl.write_text(f"{version}\n{macros}{extra}{remainder}", encoding="utf-8")
        kind = {".vsh": "vert", ".fsh": "frag", ".csh": "comp"}[stage.suffix]
        result = subprocess.run(
            [compiler, "-S", kind, str(glsl)],
            capture_output=True,
            text=True,
            check=False,
        )
        if result.returncode:
            diagnostic = (result.stdout + result.stderr).strip()
            failures.append(f"{stage.relative_to(root)}:\n{diagnostic}")
    return len(stages), len(failures), failures


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", default="glslangValidator", help="glslangValidator executable")
    args = parser.parse_args()
    compiler = shutil.which(args.compiler)
    if compiler is None:
        parser.error(f"{args.compiler!r} was not found; install glslang-tools (for example: apt install glslang-tools)")
    if not SHADERS.is_dir():
        parser.error(f"Shader source tree not found: {SHADERS}")

    with tempfile.TemporaryDirectory(prefix="hyperrealism-shader-") as temporary:
        work = Path(temporary)
        cases = (
            ("default", (), False),
            (
                "pbr-off-style-max",
                (("#define PBR_MODE 1", "#define PBR_MODE 0"),
                 ("#define STYLE_STRENGTH 70", "#define STYLE_STRENGTH 100")),
                False,
            ),
            ("feature-matrix-dh", FEATURE_CHANGES, True),
        )
        total = compiled = failures = 0
        diagnostics: list[str] = []
        for label, changes, with_dh in cases:
            root = work / label / "shaders"
            output = work / "compiled"
            output.mkdir(exist_ok=True)
            prepare_variant(root, changes)
            count, failed, messages = compile_variant(label, root, compiler, output, with_dh)
            total += count
            compiled += count - failed
            failures += failed
            diagnostics.extend(f"{label}: {message}" for message in messages)
            print(f"{label}: {count - failed}/{count} stages compiled")
        if diagnostics:
            print("\n".join(diagnostics), file=sys.stderr)
        print(f"Iris 26.3 GLSL validation: {compiled}/{total} stages compiled successfully.")
        return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
