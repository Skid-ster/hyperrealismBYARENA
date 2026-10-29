# Downloads the newest published preview of this project, checks its SHA-256,
# and installs it in Minecraft's resourcepacks directory. No code is executed
# from the downloaded archive.
[CmdletBinding()]
param(
    [string]$Destination = (Join-Path $env:APPDATA '.minecraft\resourcepacks'),
    [switch]$Force
)
$ErrorActionPreference = 'Stop'
$packName = 'HyperRealism128-MC26.3-EXPERIMENTAL.zip'
$hashName = "$packName.sha256"
$api = 'https://api.github.com/repos/Skid-ster/hyperrealismBYARENA/releases?per_page=100'
$prefix = 'https://github.com/Skid-ster/hyperrealismBYARENA/releases/download/'
$target = Join-Path $Destination $packName
if ((Test-Path -LiteralPath $target) -and -not $Force) {
    throw "The preview already exists at $target. Rerun with -Force to replace it."
}
# GitHub's API requires a User-Agent. Windows PowerShell 5.1 needs TLS 1.2.
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$releases = Invoke-RestMethod -Uri $api -Headers @{ 'User-Agent' = 'HyperRealism128-preview-installer' }
$release = @($releases | Where-Object {
    -not $_.draft -and @($_.assets | Where-Object { $_.name -eq $packName }).Count -eq 1 -and
    @($_.assets | Where-Object { $_.name -eq $hashName }).Count -eq 1
} | Sort-Object published_at -Descending | Select-Object -First 1)
if ($release.Count -ne 1) { throw 'No published preview with a ZIP and SHA-256 was found.' }
$packAsset = @($release[0].assets | Where-Object { $_.name -eq $packName })[0]
$hashAsset = @($release[0].assets | Where-Object { $_.name -eq $hashName })[0]
foreach ($asset in @($packAsset, $hashAsset)) {
    if (-not $asset.browser_download_url.StartsWith($prefix, [StringComparison]::Ordinal)) {
        throw 'Refusing an unexpected download host or repository.'
    }
}
$temp = Join-Path ([IO.Path]::GetTempPath()) ("hyperrealism-" + [Guid]::NewGuid().ToString('N'))
try {
    New-Item -ItemType Directory -Path $temp | Out-Null
    $zip = Join-Path $temp $packName
    $checksum = Join-Path $temp $hashName
    Invoke-WebRequest -Uri $packAsset.browser_download_url -OutFile $zip -UseBasicParsing
    Invoke-WebRequest -Uri $hashAsset.browser_download_url -OutFile $checksum -UseBasicParsing
    $line = (Get-Content -LiteralPath $checksum -Raw).Trim()
    if ($line -notmatch '^([0-9a-fA-F]{64})\s+\*?HyperRealism128-MC26\.3-EXPERIMENTAL\.zip$') {
        throw 'The preview checksum file has an unexpected format.'
    }
    $expected = $Matches[1]
    $actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $zip).Hash
    if ($actual -ine $expected) { throw 'SHA-256 mismatch; the existing pack was not changed.' }
    New-Item -ItemType Directory -Path $Destination -Force | Out-Null
    # Recheck immediately before copying; without -Force never replace an existing pack.
    if ((Test-Path -LiteralPath $target) -and -not $Force) { throw "The preview already exists at $target." }
    Copy-Item -LiteralPath $zip -Destination $target -Force:$Force
    Write-Host "Installed experimental preview $($release[0].tag_name): $target"
    Write-Host 'Enable it in Minecraft Resource Packs. A compatible shader is required for PBR.'
} finally {
    if (Test-Path -LiteralPath $temp) { Remove-Item -LiteralPath $temp -Recurse -Force }
}
