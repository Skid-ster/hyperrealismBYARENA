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
    chinese = (SHADERS / "lang/zh_CN.lang").read_text(encoding="utf-8")
    material = (SHADERS / "lib/common/material.glsl").read_text(encoding="utf-8")
    readme = (PACK / "README.md").read_text(encoding="utf-8")
    license_text = (PACK / "LICENSE").read_text(encoding="utf-8")

    require("#define PBR_MODE 1" in settings, "LabPBR 1.3 must be enabled by default")
    require("#define STYLE_STRENGTH 70" in settings, "Expected stylized default is missing")
    require("STYLE_STRENGTH" in properties and "STYLE_STRENGTH" in english and "STYLE_STRENGTH" in chinese,
            "Style control must be visible and labeled in both supported Iris locales")
    require("hyperrealismGrade" in processing and "coolShadow" in processing
            and "warmHighlight" in processing, "Teal/amber grade is incomplete")
    require("#define tonemap agxTonemap" in tonemap, "Expected AgX default is missing")
    require("materialFromLabPBR" in material and "specularData" in material,
            "LabPBR material parsing is missing")
    require("#if PBR_MODE == 1\n#define HYPERREALISM_LAB_PBR 1" in material,
            "Explicit LabPBR mode must decode sidecars without depending on a stale global format flag")
    require("MC_TEXTURE_FORMAT_LAB_PBR" in material and "PBR Activity" in english,
            "LabPBR format flag must remain visible in the PBR Activity diagnostic")
    require("labPBRNormal" in material and "normalData.b" in material and "normalData.a" in material,
            "LabPBR normal/AO/height channel separation is missing")
    require("f0Byte < 230" in material and "f0Byte < 238" in material and "OTHER_METAL" in material,
            "LabPBR dielectric/predefined-metal/other-metal ranges are missing")
    require("getMetalF0" in material, "Predefined LabPBR metals need spectral F0, not albedo double-tinting")
    require(all(f"#define {name} {value}u" in material for name, value in [
        ("NO_METAL", 0), ("IRON", 1), ("GOLD", 2), ("ALUMINIUM", 3), ("CHROME", 4),
        ("COPPER", 5), ("LEAD", 6), ("PLATINUM", 7), ("SILVER", 8), ("OTHER_METAL", 9),
    ]), "LabPBR metal IDs must be unsigned literals for strict switch(uint) compilers")
    require("blueByte <= 64" in material and "blueByte - 64" in material,
            "LabPBR porosity/SSS multiplexing is missing")
    require("emissionByte == 255" in material, "LabPBR no-emission sentinel is missing")
    require("#define PBR_NORMAL_STRENGTH 1.0" in settings and "#define PBR_AO_STRENGTH 1.0" in settings
            and "#define PBR_DEBUG 0" in settings and "#define PARALLAX\n" in settings, "PBR controls/debug/POM defaults are missing")
    require("PBR_NORMAL_STRENGTH" in properties and "PBR_AO_STRENGTH" in properties and "PBR_DEBUG" in properties,
            "PBR controls must be visible in Iris settings")
    for locale in (english, chinese):
        require("option.PBR_NORMAL_STRENGTH" in locale and "option.PBR_DEBUG" in locale,
                "PBR controls must be labeled in both supported Iris locales")
    for name in ("gbuffers_main.glsl", "gbuffers_clrwl.glsl"):
        gbuffer = (SHADERS / "program" / name).read_text(encoding="utf-8")
        require("materialFromLabPBR(albedo.rgb, normalData, specularData)" in gbuffer,
                f"{name} must use the shared LabPBR decoder")
        require("vec3 mappedNormal = texture(normals, texcoord).rgb;" not in gbuffer,
                f"{name} still contains dead/incorrect entity normal code")
        require("pbrDebugColor(PBR_DEBUG, normalData, specularData, material)" in gbuffer,
                f"{name} must expose the PBR channel debug view")
    require("6a591097b6348155a4ba5a1aab92b79a2c9896d3" in readme,
            "Upstream source revision must remain pinned in the shader notes")
    require("MIT License" in license_text and "Copyright (c) 2024 Josh Britain" in license_text,
            "Upstream MIT notice must be retained")
    require((PACK / "UPSTREAM_GLIMMER_README.md").is_file()
            and (PACK / "UPSTREAM_GLIMMER_DEVELOPMENT.md").is_file()
            and (SHADERS / "textures/LICENSE-MIT_tony_mc_mapface").is_file(),
            "Upstream acknowledgements, development notes, and texture license must be retained")
    files = [path for path in SHADERS.rglob("*") if path.is_file()]
    require(len(files) >= 300, f"Shader source tree looks incomplete: {len(files)} files")
    print(f"Shader structure and MIT provenance checks passed ({len(files)} source/runtime files).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
