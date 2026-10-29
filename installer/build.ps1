# Build and test with the .NET Framework already present on Windows 11.
# No NuGet packages or installation framework are required.
$ErrorActionPreference = 'Stop'
$csc = Join-Path $env:WINDIR 'Microsoft.NET\Framework64\v4.0.30319\csc.exe'
if (-not (Test-Path -LiteralPath $csc)) { throw 'The .NET Framework C# compiler was not found.' }
$root = Split-Path -Parent $PSScriptRoot
$out = Join-Path $root 'installer-output'
New-Item -ItemType Directory -Force -Path $out | Out-Null
$common = @('/nologo', '/codepage:65001', '/platform:anycpu', '/optimize+', '/reference:System.Core.dll', '/reference:System.Web.Extensions.dll')
& $csc @common '/target:winexe' "/resource:$PSScriptRoot\assets\prism-grass.png,HyperRealismInstaller.Grass.png" "/resource:$PSScriptRoot\assets\ATTRIBUTION.txt,HyperRealismInstaller.IconCredits.txt" '/reference:System.Drawing.dll' '/reference:System.Windows.Forms.dll' "/out:$out\HyperRealism128-Installer.exe" "$PSScriptRoot\PackInstaller.cs" "$PSScriptRoot\SelfUpdater.cs" "$PSScriptRoot\LauncherIntegration.cs" "$PSScriptRoot\Program.cs"
if ($LASTEXITCODE -ne 0) { throw 'GUI installer compilation failed.' }
& $csc @common '/target:exe' '/reference:System.Windows.Forms.dll' "/out:$out\InstallerTests.exe" "$PSScriptRoot\PackInstaller.cs" "$PSScriptRoot\SelfUpdater.cs" "$PSScriptRoot\LauncherIntegration.cs" "$PSScriptRoot\Tests.cs"
if ($LASTEXITCODE -ne 0) { throw 'Installer test compilation failed.' }
& "$out\InstallerTests.exe" "$out\HyperRealism128-Installer.exe"
if ($LASTEXITCODE -ne 0) { throw 'Installer backend tests failed.' }
$smoke = Start-Process -FilePath "$out\HyperRealism128-Installer.exe" -ArgumentList '--smoke-test' -Wait -PassThru
if ($smoke.ExitCode -ne 0) {
    $errorFile = Join-Path $env:TEMP 'hyperrealism-gui-smoke-error.txt'
    if (Test-Path -LiteralPath $errorFile) { Write-Host (Get-Content -LiteralPath $errorFile -Raw) }
    throw "GUI construction smoke test failed: $($smoke.ExitCode)"
}
# Verify that the full window can be rendered, without doing update checks.
$image = Join-Path $out 'layout-smoke.png'
$render = Start-Process -FilePath "$out\HyperRealism128-Installer.exe" -ArgumentList @('--capture-ui', "`"$image`"") -Wait -PassThru
if ($render.ExitCode -ne 0 -or -not (Test-Path -LiteralPath $image) -or (Get-Item -LiteralPath $image).Length -lt 1000) { throw 'GUI render smoke test failed.' }
Remove-Item -LiteralPath $image -Force
Write-Host 'GUI build and smoke tests passed.'
