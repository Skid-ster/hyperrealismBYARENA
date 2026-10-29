# Build and test with the .NET Framework already present on Windows 11.
# No NuGet packages or installation framework are required.
$ErrorActionPreference = 'Stop'
$csc = Join-Path $env:WINDIR 'Microsoft.NET\Framework64\v4.0.30319\csc.exe'
if (-not (Test-Path -LiteralPath $csc)) { throw 'The .NET Framework C# compiler was not found.' }
$root = Split-Path -Parent $PSScriptRoot
$out = Join-Path $root 'installer-output'
New-Item -ItemType Directory -Force -Path $out | Out-Null
$common = @('/nologo', '/platform:anycpu', '/optimize+', '/reference:System.Core.dll', '/reference:System.Web.Extensions.dll')
& $csc @common '/target:winexe' '/reference:System.Drawing.dll' '/reference:System.Windows.Forms.dll' "/out:$out\HyperRealism128-Installer.exe" "$PSScriptRoot\PackInstaller.cs" "$PSScriptRoot\SelfUpdater.cs" "$PSScriptRoot\Program.cs"
if ($LASTEXITCODE -ne 0) { throw 'GUI installer compilation failed.' }
& $csc @common '/target:exe' '/reference:System.Windows.Forms.dll' "/out:$out\InstallerTests.exe" "$PSScriptRoot\PackInstaller.cs" "$PSScriptRoot\SelfUpdater.cs" "$PSScriptRoot\Tests.cs"
if ($LASTEXITCODE -ne 0) { throw 'Installer test compilation failed.' }
& "$out\InstallerTests.exe" "$out\HyperRealism128-Installer.exe"
if ($LASTEXITCODE -ne 0) { throw 'Installer backend tests failed.' }
$smoke = Start-Process -FilePath "$out\HyperRealism128-Installer.exe" -ArgumentList '--smoke-test' -Wait -PassThru
if ($smoke.ExitCode -ne 0) { throw "GUI construction smoke test failed: $($smoke.ExitCode)" }
Write-Host 'GUI build and smoke tests passed.'
