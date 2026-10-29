## Experimental preview — not the finished pack

This release replaces **103 block textures** for Minecraft Java 26.3, not the requested complete 280-texture pack. The rest of the game remains vanilla or uses other enabled packs. The ten reused material sets and procedural ore are experimental approximations; in-game visual quality and shader behavior have not been validated. It is a CPU-generated preview, not an RTX-optimized release. Animations, GPU acceleration, and many materials are still missing. This update adds experimental dirt, sand and sandstone textures, plus packed AO/roughness input support.

### Download and install

1. Download **HyperRealism128-MC26.3-EXPERIMENTAL.zip** below. Do not extract it.
2. On Windows, put the ZIP in `%APPDATA%\.minecraft\resourcepacks` and enable it under **Options → Resource Packs** in Minecraft Java 26.3.
3. To see PBR effects, use a compatible LabPBR shader. Support for optional AO, height, emission and subsurface channels varies by shader.

To update, download the ZIP from a newer preview release and replace the older preview ZIP. Alternatively, download and inspect `Install-HyperRealismPreview.ps1`, then run it from PowerShell; add `-Force` to replace an existing preview. It downloads the newest release containing the preview, verifies the accompanying SHA-256 file, and leaves an existing pack unchanged if verification fails. The checksum is not an independent signature.

`build_report.json` documents the input hashes and materials. Credits are included in the pack. The source commit is the current development branch; the PR remains draft because this project is incomplete.

### Validation and limitations

Automated CPU tests, release build, ZIP CRC checks, and the offline Windows installer tests passed on Linux and Windows GitHub Actions. Neither an actual Minecraft 26.3 client and shader nor the RTX 5060 Ti was available for validation. This preview is provided for testing, not as a finished or production-quality release.
