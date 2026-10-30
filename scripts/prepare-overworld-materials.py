#!/usr/bin/env python3
"""Bake CC0 foliage atlas and a biome-neutral grass-side cutout.

Usage: python scripts/prepare-overworld-materials.py tree-canopy.glb
Requires Pillow and numpy; the 1K tree asset and its provenance are documented
in work/ASSET_CREDITS.md. This is an offline authoring tool, not a build step.
"""
import hashlib
import io
import json
import math
import random
import struct
import sys
from pathlib import Path

import numpy as np
from PIL import Image

root = Path('work/materials')
glb = Path(sys.argv[1]).read_bytes()
if hashlib.sha256(glb).hexdigest() != '9eb662fd339645584a69f176643dca8f7d2a16e2daff6ca1b7049b0b9ed6b747':
    raise SystemExit('Unexpected tree GLB hash')
length = struct.unpack_from('<I', glb, 12)[0]
meta = json.loads(glb[20:20 + length])
base = 20 + length + 8

def image(name):
    entry = next(i for i in meta['images'] if i['name'] == name)
    view = meta['bufferViews'][entry['bufferView']]
    off = base + view.get('byteOffset', 0)
    return Image.open(io.BytesIO(glb[off:off + view['byteLength']])).copy()

source = image('tree_small_02_leaves_diff-tree_small_02_leaves_alpha').convert('RGBA')
normal = image('tree_small_02_leaves_nor_gl').convert('RGB')
rough = image('tree_small_02_leaves_rough').convert('RGB').getchannel('G')
# Three separate photographed leaf fragments, selected from the source atlas.
# Rectangles are specified in an 800px preview coordinate system.
rects = [(45, 39, 155, 117), (199, 334, 327, 414), (197, 420, 331, 500)]
rng = random.Random(26032026)
size = 1024
color_out = Image.new('RGBA', (size, size))
normal_out = Image.new('RGB', (size, size), (128, 128, 255))
rough_out = Image.new('L', (size, size), 235)
for row in range(8):
    for col in range(8):
        rect = rects[rng.randrange(len(rects))]
        box = tuple(round(v * 1024 / 800) for v in rect)
        c, n, r = source.crop(box), normal.crop(box), rough.crop(box)
        # Mask away neighboring atlas fragments outside the selected leaf.
        yy, xx = np.ogrid[-1:1:complex(c.height), -1:1:complex(c.width)]
        oval = np.clip((1.03 - xx * xx - yy * yy) * 20, 0, 1)
        alpha = np.asarray(c.getchannel('A'), dtype=np.float32) * oval
        gray = c.convert('L')
        c = Image.merge('RGBA', (gray, gray, gray, Image.fromarray(alpha.astype(np.uint8))))
        angle = rng.uniform(-55, 55)
        scale = rng.uniform(1.04, 1.34)
        dims = (round(c.width * scale), round(c.height * scale))
        c = c.resize(dims, Image.Resampling.LANCZOS).rotate(angle, Image.Resampling.BICUBIC, expand=True)
        n = n.resize(dims, Image.Resampling.BILINEAR).rotate(angle, Image.Resampling.BILINEAR, expand=True, fillcolor=(128,128,255))
        r = r.resize(dims, Image.Resampling.BILINEAR).rotate(angle, Image.Resampling.BILINEAR, expand=True, fillcolor=235)
        # Rotate GL tangent-space X/Y to follow each rotated photo fragment.
        arr = np.asarray(n, dtype=np.float32).copy()
        x, y = (arr[:,:,0] - 127.5) / 127.5, (arr[:,:,1] - 127.5) / 127.5
        theta = math.radians(angle)
        arr[:,:,0] = (np.cos(theta)*x - np.sin(theta)*y)*127.5 + 127.5
        arr[:,:,1] = (np.sin(theta)*x + np.cos(theta)*y)*127.5 + 127.5
        n = Image.fromarray(np.clip(arr,0,255).astype(np.uint8), 'RGB')
        x0 = round((col + .5)*size/8 + rng.uniform(-28,28) - c.width/2)
        y0 = round((row + .5)*size/8 + rng.uniform(-28,28) - c.height/2)
        for dx in (-size, 0, size):
            for dy in (-size, 0, size):
                pos = (x0 + dx, y0 + dy)
                color_out.alpha_composite(c, pos)
                normal_out.paste(n, pos, c.getchannel('A'))
                rough_out.paste(r, pos, c.getchannel('A'))
leafdir = root/'tree_leaves'; leafdir.mkdir(exist_ok=True)
color_out.save(leafdir/'neutral_color.png')
normal_out.save(leafdir/'normal.png')
rough_out.save(leafdir/'roughness.png')

# A cutout fringe over the separate, untinted soil side. All waves repeat in X.
grass = Image.open(root/'leafy_grass/neutral_color.png').convert('RGBA')
y, x = np.ogrid[:size, :size]
phase = 2 * np.pi * x / size
edge = 154 + 22*np.sin(phase*5+.3) + 14*np.sin(phase*13+.8) + 8*np.sin(phase*29+2.1)
# Individual small blades extend below the connected top fringe.
blades = np.maximum(0, np.sin(phase*37 + .4))**12 * 72
edge = edge + blades
mask = np.clip((edge-y)*1.8 + 128, 0, 255).astype(np.uint8)
grass.putalpha(Image.fromarray(mask, 'L'))
grass.save(root/'leafy_grass/side_overlay.png')
print('Prepared foliage and grass side overlay')
