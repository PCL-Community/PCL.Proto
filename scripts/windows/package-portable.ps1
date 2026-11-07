param([string]$Profile = 'release')
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = (Resolve-Path "$PSScriptRoot/../..").Path
$version = (Get-Content "$root/package.json" -Raw | ConvertFrom-Json).version
$buildRoot = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { "$root/target" }
$build = Join-Path $buildRoot $Profile
$packageName = "PCL.Proto_${version}_windows_x64"
if ($Profile -ne 'release') { $packageName += "_$Profile" }
$portableRoot = "$root/target/portable"
$stagingRoot = Join-Path $portableRoot ('.staging-' + [guid]::NewGuid().ToString('N'))
$destination = Join-Path $stagingRoot $packageName
New-Item -ItemType Directory -Path $destination -Force | Out-Null
Copy-Item "$build/pcl-proto.exe" "$destination/PCL.Proto.exe"
Copy-Item "$build/WinDivert64.sys" $destination
Copy-Item "$PSScriptRoot/install-runtime.ps1", "$PSScriptRoot/Install-Dependencies.cmd", "$PSScriptRoot/Start.cmd" $destination
$license = $env:WINDIVERT_LICENSE
if (-not $license) { $license = "$root/target/build-tools/windivert-2.2.2/WinDivert-2.2.2-A/LICENSE" }
Copy-Item $license "$destination/WinDivert-LICENSE.txt"
@'
PCL.Proto portable edition (Windows x64)

Extract the entire directory and run Start.cmd. It checks Npcap (including its
driver and WinPcap API-compatible mode) and WebView2,
downloads missing prerequisites from their official publishers, and opens their
installers. Npcap requires administrator permission and an interactive installation.
Afterward, it opens PCL.Proto.exe. Once dependencies are installed, you can also
run the EXE directly. Install-Dependencies.cmd checks/installs prerequisites only.
The launcher itself needs no installation.
Keep WinDivert64.sys beside the executable; it is required for EasyTier networking.

Npcap is not included: its free license does not grant redistribution rights.
Npcap license: https://github.com/nmap/npcap/blob/master/LICENSE
WinDivert source and license: https://github.com/basil00/WinDivert/tree/v2.2.2
WebView2 Runtime: https://developer.microsoft.com/microsoft-edge/webview2/
'@ | Set-Content "$destination/README.txt" -Encoding utf8
$archive = Join-Path $portableRoot "$packageName.zip"
Compress-Archive -Path $destination -DestinationPath $archive -Force
Write-Host "Portable package: $archive"
