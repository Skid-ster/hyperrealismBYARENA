#!/usr/bin/env python3
"""Recreate the biome-neutral albedo from the pinned CC0 mirror (Pillow needed)."""
import hashlib
import sys
from pathlib import Path

from PIL import Image

source = Path(sys.argv[1])
expected = "cfa40bc9d9417d1852db8753a8d5917f110c40101179f63543c382e39bc05e4a"
if hashlib.sha256(source.read_bytes()).hexdigest() != expected:
    raise SystemExit("Unexpected source hash; use the pinned meadow-color.jpg")
output = Path("work/materials/leafy_grass/neutral_color.png")
with Image.open(source) as image:
    image.convert("L").convert("RGB").save(output)
print(f"Saved {output}")
