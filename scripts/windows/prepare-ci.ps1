$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if ($env:GITHUB_ACTIONS -ne 'true') { throw 'This script prepares an ephemeral GitHub Actions runner. For local development use bun run setup:windows.' }

$directory = Join-Path $env:RUNNER_TEMP 'pcl-windows-dependencies'
New-Item -ItemType Directory -Path $directory -Force | Out-Null
function Get-CheckedFile($Url, $Name, $Hash) {
    $file = Join-Path $directory $Name
    Invoke-WebRequest -Uri $Url -OutFile $file
    if ((Get-FileHash $file -Algorithm SHA256).Hash -ne $Hash) { throw "Checksum mismatch: $Name" }
    return $file
}

$windivert = Get-CheckedFile 'https://github.com/basil00/WinDivert/releases/download/v2.2.2/WinDivert-2.2.2-A.zip' 'windivert.zip' '63CB41763BB4B20F600B6DE04E991A9C2BE73279E317D4D82F237B150C5F3F15'
Expand-Archive $windivert -DestinationPath (Join-Path $directory 'windivert') -Force
$windivertRoot = (Resolve-Path (Join-Path $directory 'windivert/WinDivert-2.2.2-A')).Path
"WINDIVERT_DRIVER=$windivertRoot\x64\WinDivert64.sys" >> $env:GITHUB_ENV
"WINDIVERT_LICENSE=$windivertRoot\LICENSE" >> $env:GITHUB_ENV
