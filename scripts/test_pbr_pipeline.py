#!/usr/bin/env python3
"""Headless PBR pipeline checks for a generated HyperRealism pack.

CPU checks validate the published LabPBR byte contract and archive paths. When
ModernGL/EGL is available, the script also compiles the *actual* material.glsl
and runs it against generated _n/_s maps, reading back decoded normals,
roughness, metal IDs, emission, SSS and the PBR Activity debug view. By default
it sweeps every generated block triple, one CTM tile per connected-glass
directory, and every entity triple. This is software OpenGL validation, not an
Iris/Minecraft runtime test.
"""
from __future__ import annotations

import argparse
import array
import json
import math
import sys
import traceback
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SHADER = ROOT / "shaderpacks/HyperRealism-Stylized-PBR/shaders"


def fail(message: str) -> None:
    raise AssertionError(message)


def load_rgba(path: Path) -> Image.Image:
    if not path.is_file():
        fail(f"missing texture: {path}")
    return Image.open(path).convert("RGBA")


def channel_stats(image: Image.Image, channel: int) -> dict[str, float]:
    values = image.getchannel(channel).tobytes()
    return {
        "min": min(values),
        "max": max(values),
        "mean": sum(values) / len(values),
    }


def cpu_checks(pack: Path) -> dict:
    block = pack / "assets/minecraft/textures/block"
    entity = pack / "assets/minecraft/textures/entity"
    ctm = pack / "assets/minecraft/optifine/ctm"
    texture_properties = pack / "assets/minecraft/optifine/texture.properties"
    if not texture_properties.is_file():
        fail("missing assets/minecraft/optifine/texture.properties")
    if "format=lab-pbr/1.3" not in texture_properties.read_text():
        fail("texture.properties does not declare lab-pbr/1.3")

    checked = 0
    for base in sorted(block.glob("*.png")):
        stem = base.name[:-4]
        if stem.endswith("_n") or stem.endswith("_s"):
            continue
        normal = block / f"{stem}_n.png"
        specular = block / f"{stem}_s.png"
        a, n, s = load_rgba(base), load_rgba(normal), load_rgba(specular)
        if a.size != n.size or a.size != s.size:
            fail(f"size mismatch for {stem}: {a.size} {n.size} {s.size}")
        checked += 1

    ctm_tiles = 0
    for directory in sorted(ctm.glob("*")):
        if not directory.is_dir():
            continue
        for index in range(5):
            tile = directory / f"{index}.png"
            a, n, s = (
                load_rgba(tile),
                load_rgba(directory / f"{index}_n.png"),
                load_rgba(directory / f"{index}_s.png"),
            )
            if a.size != n.size or a.size != s.size:
                fail(f"CTM size mismatch: {tile}")
            ctm_tiles += 1

    entity_triples = 0
    for base in sorted(entity.rglob("*.png")):
        stem = base.with_suffix("")
        if stem.name.endswith("_n") or stem.name.endswith("_s"):
            continue
        load_rgba(base)
        load_rgba(Path(f"{stem}_n.png"))
        load_rgba(Path(f"{stem}_s.png"))
        entity_triples += 1

    expectations = {
        "iron_block": {"g": 230},
        "gold_block": {"g": 231},
        "glowstone": {"emissive": True},
        "redstone_lamp_on": {"emissive": True},
        "stone": {"emissive": False},
        "oak_leaves": {"sss": True},
    }
    for name, expected in expectations.items():
        specular = load_rgba(block / f"{name}_s.png")
        g = channel_stats(specular, 1)
        alpha = channel_stats(specular, 3)
        blue = channel_stats(specular, 2)
        if "g" in expected and (g["min"] != expected["g"] or g["max"] != expected["g"]):
            fail(f"{name}: expected constant LabPBR metal/F0 G={expected['g']}, got {g}")
        if expected.get("emissive") and alpha["max"] >= 255:
            fail(f"{name}: emissive map must stay below the 255 no-emission sentinel")
        if expected.get("emissive") is False and alpha["min"] != 255:
            fail(f"{name}: non-emissive map must use alpha=255 sentinel")
        if expected.get("sss") and blue["max"] <= 64:
            fail(f"{name}: expected subsurface byte >64, got {blue}")

    return {
        "block_triples": checked,
        "ctm_tiles": ctm_tiles,
        "entity_triples": entity_triples,
        "labpbr_format": "lab-pbr/1.3",
    }


def gl_checks(pack: Path, shader_root: Path, sweep: bool = True) -> dict:
    try:
        import moderngl
    except ImportError:
        return {"skipped": "moderngl is not installed"}

    ctx = None
    errors = []
    for backend in ("egl", None):
        try:
            ctx = (
                moderngl.create_standalone_context(backend=backend)
                if backend
                else moderngl.create_standalone_context()
            )
            break
        except Exception as error:
            errors.append(f"{backend or 'default'}: {type(error).__name__}: {error}")
    if ctx is None:
        return {"skipped": "no headless OpenGL context: " + "; ".join(errors)}

    material_source = (shader_root / "lib/common/material.glsl").read_text()
    vertex = """
    #version 330
    in vec2 in_pos;
    out vec2 uv;
    void main() {
        uv = in_pos * 0.5 + 0.5;
        gl_Position = vec4(in_pos, 0.0, 1.0);
    }
    """
    fragment = (
        """
    #version 330
    #define PBR_MODE 1
    #define PBR_NORMAL_STRENGTH 1.0
    #define PBR_AO_STRENGTH 1.0
    uniform sampler2D albedoTex;
    uniform sampler2D normals;
    uniform sampler2D specular;
    in vec2 uv;
    layout(location = 0) out vec4 outNormalHeight;
    layout(location = 1) out vec4 outRoughF0EmissionAo;
    layout(location = 2) out vec4 outSssPorosityMetal;
    layout(location = 3) out vec4 outDebugLight;
    float pow2(float x) { return x * x; }
    float clamp01(float x) { return clamp(x, 0.0, 1.0); }
    """
        + material_source
        + """
    void main() {
        vec4 n = texture(normals, uv);
        vec4 s = texture(specular, uv);
        vec3 a = texture(albedoTex, uv).rgb;
        Material m = materialFromLabPBR(a, n, s);
        vec3 mapped = labPBRNormal(n, mat3(1.0));
        outNormalHeight = vec4(mapped, n.a);
        outRoughF0EmissionAo = vec4(m.roughness, m.f0.r, m.emission, m.ao);
        outSssPorosityMetal = vec4(m.sss, m.porosity, float(m.metalID), m.f0.g);
        outDebugLight = vec4(
            pbrDebugColor(9, n, s, m),
            dot(mapped, normalize(vec3(0.4, 0.3, 0.85)))
        );
    }
    """
    )
    try:
        program = ctx.program(vertex_shader=vertex, fragment_shader=fragment)
    except Exception as error:
        fail(
            "headless GLSL compile/link failed on "
            f"{ctx.info.get('GL_RENDERER', 'unknown')} / OpenGL {ctx.version_code}: {error}"
        )
    size = 64
    attachments = [ctx.texture((size, size), 4, dtype="f4") for _ in range(4)]
    fbo = ctx.framebuffer(color_attachments=attachments)
    vbo = ctx.buffer(array.array("f", [-1, -1, 3, -1, -1, 3]).tobytes())
    vao = ctx.vertex_array(program, [(vbo, "2f", "in_pos")])
    block = pack / "assets/minecraft/textures/block"
    ctm = pack / "assets/minecraft/optifine/ctm"
    entity = pack / "assets/minecraft/textures/entity"

    def make_texture(path: Path | None, fallback: tuple[int, int, int, int]):
        if path is None:
            data = bytes(fallback) * (size * size)
            texture = ctx.texture((size, size), 4, data)
        else:
            image = load_rgba(path)
            texture = ctx.texture(image.size, 4, image.tobytes())
        texture.filter = (ctx.NEAREST, ctx.NEAREST)
        texture.repeat_x = False
        texture.repeat_y = False
        return texture

    def render_paths(
        albedo_path: Path | None,
        normal_path: Path | None,
        specular_path: Path | None,
    ) -> dict:
        albedo = make_texture(albedo_path, (128, 128, 128, 255))
        normal = make_texture(normal_path, (127, 127, 255, 255))
        specular = make_texture(specular_path, (0, 0, 0, 0))
        albedo.use(0)
        normal.use(1)
        specular.use(2)
        program["albedoTex"].value = 0
        program["normals"].value = 1
        program["specular"].value = 2
        fbo.use()
        ctx.viewport = (0, 0, size, size)
        vao.render(moderngl.TRIANGLES)
        outputs = []
        for index in range(len(attachments)):
            raw = fbo.read(components=4, alignment=1, dtype="f4", attachment=index)
            outputs.append(array.array("f", raw))
        normal_out, material_out, extra_out, debug_out = outputs

        def stats(values, stride, offset):
            selected = values[offset::stride]
            return {
                "min": min(selected),
                "max": max(selected),
                "mean": sum(selected) / len(selected),
            }

        result = {
            "roughness": stats(material_out, 4, 0),
            "f0_r": stats(material_out, 4, 1),
            "emission": stats(material_out, 4, 2),
            "ao": stats(material_out, 4, 3),
            "sss": stats(extra_out, 4, 0),
            "porosity": stats(extra_out, 4, 1),
            "metal_id": stats(extra_out, 4, 2),
            "debug_r": stats(debug_out, 4, 0),
            "debug_g": stats(debug_out, 4, 1),
            "ndotl": stats(debug_out, 4, 3),
            "normal_x": stats(normal_out, 4, 0),
            "normal_y": stats(normal_out, 4, 1),
        }
        for texture in (albedo, normal, specular):
            texture.release()
        return result

    def render_block(name: str, use_normal: bool = True, use_specular: bool = True) -> dict:
        return render_paths(
            block / f"{name}.png",
            block / f"{name}_n.png" if use_normal else None,
            block / f"{name}_s.png" if use_specular else None,
        )

    named = (
        "stone",
        "cobblestone",
        "oak_planks",
        "iron_block",
        "gold_block",
        "glowstone",
        "oak_leaves",
        "redstone_lamp_on",
        "diamond_block",
        "emerald_block",
        "lapis_block",
        "glass",
        "coal_block",
        "redstone_block",
    )
    cases = {name: render_block(name) for name in named}
    missing = render_block("stone", use_normal=False, use_specular=False)
    normal_only = render_block("stone", use_normal=True, use_specular=False)
    specular_only = render_block("stone", use_normal=False, use_specular=True)

    def close(value: float, target: float, tolerance: float = 0.02) -> bool:
        return abs(value - target) <= tolerance

    def assert_green(name: str, data: dict) -> None:
        if data["debug_g"]["mean"] < 0.9 or data["debug_r"]["mean"] > 0.1:
            fail(f"{name}: PBR Activity did not report active LabPBR specular data: {data}")

    if not close(cases["iron_block"]["metal_id"]["mean"], 1.0):
        fail(f"iron_block did not decode as LabPBR iron: {cases['iron_block']}")
    if not close(cases["gold_block"]["metal_id"]["mean"], 2.0):
        fail(f"gold_block did not decode as LabPBR gold: {cases['gold_block']}")
    if cases["glowstone"]["emission"]["mean"] <= 0.1:
        fail(f"glowstone emission did not decode: {cases['glowstone']}")
    if cases["redstone_lamp_on"]["emission"]["mean"] <= 0.3:
        fail(f"redstone_lamp_on emission did not decode: {cases['redstone_lamp_on']}")
    if cases["oak_leaves"]["sss"]["mean"] <= 0.1:
        fail(f"oak_leaves subsurface did not decode: {cases['oak_leaves']}")
    for name in ("diamond_block", "emerald_block", "lapis_block", "glass"):
        data = cases[name]
        assert_green(name, data)
        if data["metal_id"]["mean"] > 0.01:
            fail(f"{name}: gem/glass should be dielectric: {data}")
        if data["roughness"]["mean"] > 0.35:
            fail(f"{name}: gem/glass should be smooth: {data}")
    for name in ("coal_block", "redstone_block"):
        assert_green(name, cases[name])
        if cases[name]["roughness"]["mean"] <= 0.05:
            fail(f"{name}: expected a rough dielectric response: {cases[name]}")
    for name in ("stone", "cobblestone", "iron_block", "gold_block"):
        data = cases[name]
        assert_green(name, data)
        if data["ndotl"]["max"] - data["ndotl"]["min"] < 0.005:
            fail(f"{name}: mapped normal produced no lighting variation: {data}")
    if missing["debug_r"]["mean"] < 0.9 or missing["debug_g"]["mean"] > 0.1:
        fail(f"missing-sidecar control did not report red: {missing}")
    if missing["roughness"]["mean"] < 0.99 or missing["metal_id"]["mean"] > 0.01:
        fail(f"missing-sidecar fallback was not rough dielectric: {missing}")
    if normal_only["debug_r"]["mean"] < 0.9 or not close(normal_only["debug_g"]["mean"], 0.75):
        fail(f"normal-only control did not report yellow: {normal_only}")
    assert_green("specular-only control", specular_only)
    if specular_only["normal_x"]["max"] - specular_only["normal_x"]["min"] > 0.01:
        fail(f"specular-only control should have a flat default normal: {specular_only}")

    sweep_report = {"enabled": sweep}
    if sweep:
        failures: list[str] = []
        counts = {"block": 0, "ctm": 0, "entity": 0}
        green_min = 1.0
        roughness_min, roughness_max = 1.0, 0.0
        metal_max = 0.0

        def validate(label: str, data: dict) -> None:
            nonlocal green_min, roughness_min, roughness_max, metal_max
            green_min = min(green_min, data["debug_g"]["mean"])
            roughness_min = min(roughness_min, data["roughness"]["min"])
            roughness_max = max(roughness_max, data["roughness"]["max"])
            metal_max = max(metal_max, data["metal_id"]["max"])
            problems = []
            if data["debug_g"]["mean"] < 0.9 or data["debug_r"]["mean"] > 0.1:
                problems.append("PBR Activity not green")
            for key in ("roughness", "emission", "ao", "sss", "porosity"):
                lo, hi = data[key]["min"], data[key]["max"]
                if not math.isfinite(lo) or not math.isfinite(hi) or lo < -0.001 or hi > 1.001:
                    problems.append(f"{key} out of range [{lo}, {hi}]")
            if data["metal_id"]["min"] < -0.001 or data["metal_id"]["max"] > 9.001:
                problems.append(f"metal ID out of range: {data['metal_id']}")
            for key in ("normal_x", "normal_y"):
                if data[key]["min"] < -1.001 or data[key]["max"] > 1.001:
                    problems.append(f"{key} out of range: {data[key]}")
            if problems:
                failures.append(f"{label}: {'; '.join(problems)}")

        for base in sorted(block.glob("*.png")):
            stem = base.name[:-4]
            if stem.endswith("_n") or stem.endswith("_s"):
                continue
            counts["block"] += 1
            validate(f"block/{stem}", render_block(stem))
        for directory in sorted(ctm.glob("*")):
            if not directory.is_dir():
                continue
            counts["ctm"] += 1
            validate(
                f"ctm/{directory.name}/0",
                render_paths(
                    directory / "0.png",
                    directory / "0_n.png",
                    directory / "0_s.png",
                ),
            )
        for base in sorted(entity.rglob("*.png")):
            stem = base.with_suffix("")
            if stem.name.endswith("_n") or stem.name.endswith("_s"):
                continue
            counts["entity"] += 1
            validate(
                f"entity/{stem.relative_to(entity)}",
                render_paths(base, Path(f"{stem}_n.png"), Path(f"{stem}_s.png")),
            )
        if failures:
            fail("exhaustive LabPBR sweep failed: " + " | ".join(failures[:12]))
        sweep_report.update(
            {
                "checked": counts,
                "pbr_activity_green_min": green_min,
                "roughness_range": [roughness_min, roughness_max],
                "metal_id_max": metal_max,
            }
        )

    vao.release()
    vbo.release()
    fbo.release()
    for attachment in attachments:
        attachment.release()
    program.release()
    renderer = ctx.info.get("GL_RENDERER", "unknown")
    ctx.release()
    return {
        "renderer": renderer,
        "cases": cases,
        "missing_sidecar_control": missing,
        "normal_only_control": normal_only,
        "specular_only_control": specular_only,
        "sweep": sweep_report,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=ROOT / "pilot_output/pack")
    parser.add_argument("--shader-root", type=Path, default=DEFAULT_SHADER)
    parser.add_argument("--require-gl", action="store_true")
    parser.add_argument("--skip-sweep", action="store_true", help="only run named controls")
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()

    report = {"cpu": cpu_checks(args.pack)}
    report["gl"] = gl_checks(args.pack, args.shader_root, sweep=not args.skip_sweep)
    if args.require_gl and "skipped" in report["gl"]:
        fail(f"--require-gl requested but OpenGL was unavailable: {report['gl']['skipped']}")
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, indent=2) + "\n")
    summary = {
        "block_triples": report["cpu"]["block_triples"],
        "ctm_tiles": report["cpu"]["ctm_tiles"],
        "entity_triples": report["cpu"]["entity_triples"],
        "gl": "skipped" if "skipped" in report["gl"] else report["gl"]["renderer"],
    }
    if "sweep" in report["gl"]:
        summary["sweep"] = report["gl"]["sweep"]
    print(json.dumps(summary, indent=2))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except AssertionError as error:
        print(f"PBR pipeline check failed: {error}", file=sys.stderr)
        raise SystemExit(1)
    except Exception:
        traceback.print_exc()
        raise SystemExit(1)
