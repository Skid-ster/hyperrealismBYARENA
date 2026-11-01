#!/usr/bin/env python3
"""Reconstruct conservative leaf height and cavity AO from the CC0 normal scan.

Run after prepare-overworld-materials.py. Uses only the aligned, documented
CC0-derived atlas already in work/materials/tree_leaves. Height is a Poisson
integration of its tangent-space normal slopes, not measured displacement.
Requires Pillow and NumPy; deterministic and CPU-only.
"""
from pathlib import Path

import numpy as np
from PIL import Image, ImageFilter

ROOT = Path("work/materials/tree_leaves")
color = Image.open(ROOT / "neutral_color.png").convert("RGBA")
normal = Image.open(ROOT / "normal.png").convert("RGB")
if color.size != normal.size or color.width != color.height:
    raise SystemExit("leaf albedo and normal must be aligned square atlases")

alpha = np.asarray(color.getchannel("A"), dtype=np.float64) / 255.0
n = np.asarray(normal, dtype=np.float64) / 127.5 - 1.0
# The input is OpenGL Y-up. Match material.rs' conversion to the pipeline's
# DirectX/image Y-down convention before integrating its gradient field.
nx, ny, nz = n[:, :, 0], -n[:, :, 1], np.maximum(n[:, :, 2], 0.2)
mask = np.clip((alpha - 0.03) / 0.5, 0.0, 1.0)
# Tangent-space normal slopes approximate the photographed surface gradients.
p = np.clip(-nx / nz, -1.5, 1.5) * mask
q = np.clip(-ny / nz, -1.5, 1.5) * mask

# Periodic least-squares Poisson integration keeps this atlas tileable. The
# zero-frequency term is fixed to zero; a gentle high-frequency cutoff suppresses
# pixel noise without erasing scanned veins and the central midrib.
h, w = alpha.shape
kx = 2.0 * np.pi * np.fft.fftfreq(w)[None, :]
ky = 2.0 * np.pi * np.fft.fftfreq(h)[:, None]
ph = np.fft.fft2(p)
qh = np.fft.fft2(q)
den = kx * kx + ky * ky
height_hat = np.zeros_like(ph, dtype=np.complex128)
valid = den > 0
height_hat[valid] = (-1j * kx * ph - 1j * ky * qh)[valid] / den[valid]
# A small periodic Gaussian low-pass avoids encoding scan grain as displacement.
sigma = 0.65
height_hat *= np.exp(-0.5 * den * sigma * sigma)
height = np.fft.ifft2(height_hat).real

samples = height[alpha > 0.5]
if samples.size < 32 or not np.isfinite(samples).all():
    raise SystemExit("leaf atlas has insufficient valid normal samples")
lo, hi = np.quantile(samples, [0.03, 0.97])
if hi - lo < 1e-8:
    raise SystemExit("normal scan did not produce a usable height range")
# Preserve a neutral 0.5 midplane and use the robust scanned range for clear
# relief. The pack's uniform 0.2 PBR strength compresses the packed LabPBR
# height to a restrained range around neutral.
relief = np.clip((height - (lo + hi) * 0.5) / (hi - lo), -0.5, 0.5)
height_u8 = np.clip(0.5 + relief * 0.46, 0.0, 1.0)
height_u8 = np.where(alpha > 0.01, height_u8, 0.5)
height_image = Image.fromarray(np.round(height_u8 * 255).astype(np.uint8), "L")
height_image.save(ROOT / "height.png", optimize=True)

# Very soft local concavity darkening follows reconstructed leaf valleys. This is
# intentionally conservative: clear spaces stay fully unoccluded and AO is not
# inferred from the cutout silhouette or presented as a true ray-traced bake.
blurred = np.asarray(height_image.filter(ImageFilter.GaussianBlur(radius=5)), dtype=np.float64) / 255.0
height_linear = np.asarray(height_image, dtype=np.float64) / 255.0
cavity = np.maximum(blurred - height_linear, 0.0)
ao = np.clip(1.0 - cavity * 0.9, 0.88, 1.0)
ao = np.where(alpha > 0.01, ao, 1.0)
Image.fromarray(np.round(ao * 255).astype(np.uint8), "L").save(ROOT / "ao.png", optimize=True)
print("Prepared periodic normal-derived leaf height and subtle cavity AO")
