#!/usr/bin/env python3
"""Build an original, adult temperate-chicken LabPBR entity-texture pilot.

Uses only the Python standard library. The 64x32 UV opacity footprint is encoded
as spans; no vanilla color pixels or game texture files are inputs.
"""
from __future__ import annotations

import argparse
import math
import struct
import zipfile
import zlib
from pathlib import Path

SIZE = (512, 256)  # 8x the vanilla 64x32 entity texture; preserve its aspect ratio.
SCALE = 8
# Opaque UV islands from the 26.3 adult chicken atlas, expressed as inclusive
# texel spans at native 64x32 coordinates. Empty atlas space remains transparent.
OPAQUE_SPANS = [
    [(3, 10), (16, 23), (33, 33)],
    [(3, 10), (16, 23), (32, 34)],
    [(3, 10), (14, 25), (32, 34)],
    [(0, 25), (36, 36)],
    [(0, 13), (16, 19), (36, 36)],
    [(0, 13), (16, 19), (36, 36)],
    [(0, 21), (36, 36)],
    [(0, 21), (36, 36)],
    [(0, 13)],
    *[[(6, 17)] for _ in range(4)],
    *[[(6, 17), (30, 31)] for _ in range(2)],
    *[[(0, 23), (30, 31)] for _ in range(4)],
    *[[(0, 37)] for _ in range(4)],
    *[[] for _ in range(9)],
]
assert len(OPAQUE_SPANS) == 32


def noise(ix: int, iy: int, seed: int = 31) -> float:
    n = (ix * 0x1F123BB5) ^ (iy * 0x5F356495) ^ (seed * 0x6C8E9CF5)
    n = (n ^ (n >> 13)) * 0x85EBCA6B
    n = (n ^ (n >> 16)) & 0xFFFFFFFF
    return n / 0xFFFFFFFF


def smoothstep(v: float) -> float:
    v = min(1.0, max(0.0, v))
    return v * v * (3.0 - 2.0 * v)


def value_noise(x: float, y: float, scale: float, seed: int) -> float:
    gx, gy = x / scale, y / scale
    ix, iy = math.floor(gx), math.floor(gy)
    fx, fy = smoothstep(gx - ix), smoothstep(gy - iy)
    a = noise(ix, iy, seed)
    b = noise(ix + 1, iy, seed)
    c = noise(ix, iy + 1, seed)
    d = noise(ix + 1, iy + 1, seed)
    top = a * (1.0 - fx) + b * fx
    bottom = c * (1.0 - fx) + d * fx
    return top * (1.0 - fy) + bottom * fy


def in_spans(u: int, v: int) -> bool:
    return any(lo <= u <= hi for lo, hi in OPAQUE_SPANS[v])


def region(u: int, v: int) -> str:
    # Tiny dark marks on the two head-side UVs are original, warm-brown eyes.
    if (u, v) in {(16, 1), (19, 1)}:
        return "eye"
    # Beak UVs; sculpted warm keratin rather than yellow plastic.
    if 14 <= u <= 25 and v <= 3:
        return "beak"
    # Comb and wattle UV patches, positioned independently from the feather islands.
    if ((u in (8, 9) and v == 2) or
        (16 <= u <= 19 and v in (4, 5)) or
        (14 <= u <= 21 and v in (6, 7)) or
        (u in (4, 5) and v in (7, 8))):
        return "comb"
    # Scaly yellow legs/feet UVs.
    if (32 <= u <= 34 and v <= 2) or (u == 36 and 3 <= v <= 7):
        return "leg"
    return "feather"


def feather_surface(x: int, y: int) -> tuple[tuple[int, int, int], float, float]:
    """Soft white plumage with narrow overlapping feathers and fine barbs."""
    width, height = 17.0, 27.0
    row = math.floor(y / height)
    row_y = y - row * height
    offset = (row & 1) * (width / 2.0)
    col = math.floor((x + offset) / width)
    local_x = (x + offset) - (col + 0.5) * width
    # Slightly varied feather direction prevents a rigid printed scale pattern.
    tilt = (noise(col, row, 91) - 0.5) * 0.22
    dx_px = local_x - tilt * (row_y - height * 0.5)
    t = row_y / height
    taper = 0.16 + 0.84 * math.sqrt(max(0.0, math.sin(math.pi * t)))
    half_width = width * 0.48 * taper
    across = abs(dx_px) / max(half_width, 1.0)
    edge = smoothstep((across - 0.72) / 0.28)
    inside = 1.0 - edge
    # Individual feather tone, broad down variation, and finer directional grain.
    cell_tone = noise(col, row, 101) - 0.5
    broad = value_noise(x, y, 57.0, 17) - 0.5
    medium = value_noise(x, y, 19.0, 49) - 0.5
    fine = noise(x // 2, y // 3, 73) - 0.5
    shaft = (1.0 - smoothstep((abs(dx_px) - 0.35) / 1.25)) * smoothstep(t / 0.12) * smoothstep((1.0 - t) / 0.15)
    # Fine oblique barblets are visible only close-up, as low-contrast keratin fibers.
    barb_phase = (t * 13.0 + abs(dx_px) * 0.11 + noise(col, row, 137)) % 1.0
    barbs = (1.0 - smoothstep((barb_phase - 0.12) / 0.12)) * min(1.0, abs(dx_px) / 3.0)
    shade = 0.014 * cell_tone + 0.024 * broad + 0.012 * medium + 0.010 * fine
    shade -= 0.038 * edge
    shade += 0.016 * shaft + 0.008 * barbs * inside
    # A subdued ivory base avoids chalk-white surfaces and keeps the feather detail legible.
    base = (0.925 + shade, 0.907 + shade, 0.868 + shade)
    color = tuple(max(0, min(255, round(v * 255))) for v in base)
    feather_ridge = 0.025 * inside + 0.018 * shaft + 0.008 * barbs
    height_value = 0.515 + feather_ridge + 0.006 * medium
    ao = 1.0 - 0.045 * edge
    return color, height_value, ao


def surface(kind: str, x: int, y: int) -> tuple[tuple[int, int, int], float, float, float]:
    if kind == "feather":
        color, h, ao = feather_surface(x, y)
        return color, h, ao, 0.78
    if kind == "comb":
        ridge = (0.5 + 0.5 * math.sin((x + 0.22 * y) * 0.75))
        mottling = value_noise(x, y, 13.0, 221) - 0.5
        t = max(0.0, min(1.0, 0.46 + 0.25 * ridge + 0.09 * mottling))
        color = (round(112 + 100 * t), round(20 + 33 * t), round(29 + 33 * t))
        return color, 0.54 + 0.035 * ridge, 0.92, 0.64
    if kind == "beak":
        grain = value_noise(x, y, 17.0, 310) - 0.5
        ridge = 0.5 + 0.5 * math.sin((x + 0.1 * y) * 0.38)
        shade = 0.035 * grain + 0.028 * (ridge - 0.5)
        color = (round(211 + 28 * shade), round(145 + 48 * shade), round(45 + 31 * shade))
        return color, 0.555 + 0.025 * ridge, 0.98, 0.43
    if kind == "eye":
        # Small deep brown eye with a tiny warm catchlight at high resolution.
        lx, ly = x % SCALE, y % SCALE
        if 2 <= lx <= 5 and 2 <= ly <= 5:
            if lx == 4 and ly == 2:
                return (216, 170, 88), 0.56, 1.0, 0.34
            return (53, 37, 25), 0.55, 1.0, 0.3
        return (83, 57, 34), 0.55, 1.0, 0.38
    # Legs: warm gold scales with narrow creases, not fur or feathers.
    scale_cell = ((x // 8) + (y // 6) * 2) & 1
    grain = noise(x // 4, y // 4, 407) - 0.5
    scale_shade = (-0.045 if scale_cell else 0.018) + grain * 0.018
    color = (round(210 + 27 * scale_shade), round(151 + 39 * scale_shade), round(42 + 28 * scale_shade))
    h = 0.535 + (0.022 if scale_cell else 0.0)
    return color, h, 0.96, 0.48


def to_byte(value: float) -> int:
    return max(0, min(255, round(value * 255.0)))


def png_rgba(path: Path, width: int, height: int, pixels: bytes) -> None:
    if len(pixels) != width * height * 4:
        raise ValueError("bad RGBA buffer size")
    rows = b"".join(b"\0" + pixels[y * width * 4:(y + 1) * width * 4] for y in range(height))

    def chunk(name: bytes, payload: bytes) -> bytes:
        body = name + payload
        return struct.pack(">I", len(payload)) + body + struct.pack(">I", zlib.crc32(body) & 0xFFFFFFFF)

    header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    encoded = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", zlib.compress(rows, 9)) + chunk(b"IEND", b"")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(encoded)


def build(output: Path) -> None:
    width, height = SIZE
    n = width * height
    albedo = bytearray(n * 4)
    heights = [0.5] * n
    ao = [1.0] * n
    roughness = [0.8] * n
    for y in range(height):
        v = min(31, y // SCALE)
        for x in range(width):
            u = min(63, x // SCALE)
            i = y * width + x
            kind = region(u, v)
            color, h, occlusion, rough = surface(kind, x, y)
            heights[i], ao[i], roughness[i] = h, occlusion, rough
            a = 255 if in_spans(u, v) else 0
            albedo[i * 4:i * 4 + 4] = bytes((*color, a))

    normal = bytearray(n * 4)
    specular = bytearray(n * 4)
    for y in range(height):
        for x in range(width):
            i = y * width + x
            left = heights[y * width + max(0, x - 1)]
            right = heights[y * width + min(width - 1, x + 1)]
            up = heights[max(0, y - 1) * width + x]
            down = heights[min(height - 1, y + 1) * width + x]
            nx = -(right - left) * 0.5 * 1.8
            ny = -(down - up) * 0.5 * 1.8
            inv = 1.0 / math.sqrt(nx * nx + ny * ny + 1.0)
            normal[i * 4:i * 4 + 4] = bytes((
                to_byte(nx * inv * 0.5 + 0.5),
                to_byte(ny * inv * 0.5 + 0.5),
                to_byte(ao[i]),
                max(1, to_byte(heights[i])),
            ))
            # LabPBR 1.3: R=1-sqrt(roughness), G=dielectric F0,
            # B=porosity/SSS (zero), A=255 means no emission.
            smoothness = 1.0 - math.sqrt(roughness[i])
            specular[i * 4:i * 4 + 4] = bytes((to_byte(smoothness), 10, 0, 255))

    texture_dir = output / "assets/minecraft/textures/entity/chicken"
    png_rgba(texture_dir / "chicken_temperate.png", width, height, bytes(albedo))
    png_rgba(texture_dir / "chicken_temperate_n.png", width, height, bytes(normal))
    png_rgba(texture_dir / "chicken_temperate_s.png", width, height, bytes(specular))
    (output / "pack.mcmeta").write_text(
        '{\n  "pack": {\n    "description": "HyperRealism 512 — adult chicken texture pilot",\n'
        '    "min_format": [97, 1],\n    "max_format": 97\n  }\n}\n', encoding="utf-8")
    (output / "README.txt").write_text(
        "Experimental adult temperate chicken texture pilot for Minecraft Java 26.3.\n"
        "Replaces only assets/minecraft/textures/entity/chicken/chicken_temperate.png.\n"
        "Original procedural feather artwork; vanilla UVs and entity models are unchanged.\n"
        "The _n and _s maps use LabPBR 1.3. Shader support for entity PBR maps varies.\n"
        "In-game rendering and performance are not yet validated.\n",
        encoding="utf-8",
    )
    properties = output / "assets/minecraft/optifine/texture.properties"
    properties.parent.mkdir(parents=True, exist_ok=True)
    properties.write_text("format=lab-pbr/1.3\n", encoding="utf-8")


def write_zip(pack: Path, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(destination, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for path in sorted(p for p in pack.rglob("*") if p.is_file()):
            info = zipfile.ZipInfo(path.relative_to(pack).as_posix(), date_time=(2024, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, path.read_bytes(), compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path,
                        default=Path("experiments/chicken_adult/pack"))
    parser.add_argument("--zip", type=Path, default=None,
                        help="ZIP destination (default: sibling HyperRealism-Chicken-Adult-Pilot.zip)")
    args = parser.parse_args()
    build(args.output)
    archive = args.zip or args.output.parent / "HyperRealism-Chicken-Adult-Pilot.zip"
    write_zip(args.output, archive)
    print(f"Built 512x256 adult temperate chicken LabPBR pilot in {args.output}")
    print(f"Pack ZIP: {archive}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
