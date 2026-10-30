$ErrorActionPreference = 'Stop'
$script = Join-Path $PSScriptRoot '../scripts/Install-HyperRealismPreview.ps1'
$packName = 'HyperRealism128-MC26.3-EXPERIMENTAL.zip'
$root = Join-Path ([IO.Path]::GetTempPath()) ("hr-installer-test-" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $root | Out-Null
try {
    $source = Join-Path $root 'source.zip'
    [IO.File]::WriteAllText($source, 'a test payload')
    $hash = (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant()
    $global:mockHash = "$hash  $packName"
    $global:mockSource = $source
    $global:mockCorrupt = $false
    function Invoke-RestMethod {
        param($Uri, $Headers)
        return @([pscustomobject]@{
            draft = $false; tag_name = 'v-test'; published_at = '2026-01-01T00:00:00Z';
            assets = @(
                [pscustomobject]@{name=$packName; browser_download_url="https://github.com/Skid-ster/hyperrealismBYARENA/releases/download/v-test/$packName"},
                [pscustomobject]@{name="$packName.sha256"; browser_download_url="https://github.com/Skid-ster/hyperrealismBYARENA/releases/download/v-test/$packName.sha256"}
            )
        })
    }
    function Invoke-WebRequest {
        param($Uri, $OutFile, [switch]$UseBasicParsing)
        if ($Uri.EndsWith('.sha256')) { [IO.File]::WriteAllText($OutFile, $global:mockHash) }
        elseif ($global:mockCorrupt) { [IO.File]::WriteAllText($OutFile, 'corrupt') }
        else { Copy-Item -LiteralPath $global:mockSource -Destination $OutFile }
    }
    $destination = Join-Path $root 'resourcepacks'
    . $script -Destination $destination
    $installed = Join-Path $destination $packName
    if ((Get-FileHash -LiteralPath $installed).Hash -ne (Get-FileHash -LiteralPath $source).Hash) { throw 'Installed file does not match.' }
    $blocked = $false
    try { . $script -Destination $destination } catch { $blocked = $true }
    if (-not $blocked) { throw 'Expected existing pack to be protected without -Force.' }
    $global:mockCorrupt = $true
    $blocked = $false
    try { . $script -Destination $destination -Force } catch { $blocked = $true }
    if (-not $blocked) { throw 'Expected a checksum failure.' }
    if ((Get-FileHash -LiteralPath $installed).Hash -ne (Get-FileHash -LiteralPath $source).Hash) { throw 'Checksum failure changed the installed pack.' }
    $global:mockCorrupt = $false
    . $script -Destination $destination -Force
    Write-Host 'Installer tests passed.'
} finally {
    Remove-Item -LiteralPath $root -Recurse -Force
    Remove-Variable mockHash,mockSource,mockCorrupt -Scope Global -ErrorAction SilentlyContinue
}
