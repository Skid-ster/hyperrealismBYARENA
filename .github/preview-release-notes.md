## Experimental preview — not the finished pack

This release replaces **106 block textures** for Minecraft Java 26.3, not the requested complete 280-texture pack. The rest of the game remains vanilla or uses other enabled packs. The twelve reused material sources and procedural ore are experimental approximations; in-game visual quality and shader behavior have not been validated. It is a CPU-generated preview, not an RTX-optimized release. Animations, GPU acceleration, and many materials are still missing. This update adds netherrack and two Nether ore textures based on a CC0 rock scan, clamps POM heights to at least 1, and structurally checks every 128×128 albedo/normal/specular texture triple before packaging. Ore inclusions are procedural, with finer grain and sparser distribution than the prior preview; these new appearances have not yet been checked in-game. Preview 3 replaced the previous wireframe-like cobblestone with modified photographic albedo and height inputs.

### Download and install

1. Download **HyperRealism128-MC26.3-EXPERIMENTAL.zip** below. Do not extract it.
2. On Windows, put the ZIP in `%APPDATA%\.minecraft\resourcepacks` and enable it under **Options → Resource Packs** in Minecraft Java 26.3.
3. To see PBR effects, use a compatible LabPBR shader. Support for optional AO, height, emission and subsurface channels varies by shader.

To update, download the ZIP from a newer preview release and replace the older preview ZIP. Alternatively, download and inspect `Install-HyperRealismPreview.ps1`, then run it from PowerShell; add `-Force` to replace an existing preview. It downloads the newest release containing the preview, verifies the accompanying SHA-256 file, and leaves an existing pack unchanged if verification fails. The checksum is not an independent signature.

`build_report.json` documents the input hashes and materials. Credits are included in the pack. The source commit is the current development branch; the PR remains draft because this project is incomplete.

### Validation and limitations

Automated CPU tests, release build, ZIP CRC checks, and the offline Windows installer tests passed on Linux and Windows GitHub Actions. The user reported unsatisfactory cobblestone in-game, but the active pack order was not established; this replacement has not been tested in-game. The sandbox has no actual Minecraft client, shader or RTX 5060 Ti. This preview is provided for testing, not as a finished or production-quality release.
