## Gilded Dusk shader 1 — corrected LabPBR material pipeline

This is a shader-only update for **HyperRealism: Gilded Dusk**, the optional
Iris shader for Minecraft Java 26.3. The HyperRealism resource pack is not
changed by this release.

### PBR fixes

- LabPBR decoding is now explicitly gated on both **PBR Support = LabPBR 1.3**
  and Iris detecting a resource pack that declares `format=lab-pbr/1.3`.
  Non-LabPBR packs fall back to flat normals/default material instead of being
  misread as LabPBR bytes.
- Normal maps are decoded as RG tangent XY with reconstructed Z. The blue
  channel is used only for ambient occlusion and alpha only for height/POM.
  Normal reconstruction is clamped and normalized to avoid NaNs from filtered
  or strong normals.
- Specular green is decoded by byte range: 0–229 dielectric F0, 230–237
  predefined metals, 238–255 other/albedo-tinted metal. Custom/reserved IDs no
  longer index the predefined metal table or become accidental perfect mirrors.
- Predefined metals use spectral metal F0 instead of being double-tinted by the
  block albedo.
- Specular blue now correctly multiplexes porosity (0–64) and subsurface
  scattering (65–255) as mutually exclusive channels.
- Specular alpha 255 remains the no-emission sentinel; 0–254 drives emission.
- Removed dead entity-normal code and guarded BRDF half-vector/view-angle
  denominators.

### New in-game controls

Under **Materials**:

- **Normal Map Strength** (default 1.0)
- **Material AO Strength** (default 1.0)
- **PBR Channel Debug**: Normal XY, Roughness, F0/Metal, Emission, AO, Height,
  Porosity, Subsurface. These are diagnostic views, not lighting modes.

### Install

Use Installer 17’s **Install shader** button; it selects the newest published
release containing the shader ZIP and checksum. Or download
`HyperRealism-Gilded-Dusk-MC-26.3.zip` from this release and place the ZIP itself
in the active instance’s `shaderpacks` folder. Enable the HyperRealism resource
pack, select **HyperRealism: Gilded Dusk** in Iris, and leave **PBR Support** on
**LabPBR 1.3**.

### Validation and limits

CI checks shader structure/provenance, compiles the Iris 26.3 stage matrix
including PBR-off, no-LabPBR-format fallback, feature/Distant-Horizons variants,
and a LabPBR decode harness, then builds and CRC/structure-checks the installable
ZIP. These are compiler/structural checks, **not** an in-game Minecraft/Iris GPU
render, visual-quality certification, or performance benchmark.

The current HyperRealism resource pack still uses its authored global PBR
strength of 0.2, so normal relief and smoothness are intentionally conservative.
The shader now reads those maps correctly; it does not invent missing coverage
or turn the partial pack into full vanilla PBR. Use the debug views and the
in-game checklist in `docs/PBR_VALIDATION.md` to verify channels on your machine.
