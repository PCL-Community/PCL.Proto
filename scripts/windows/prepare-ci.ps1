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

$sdk = Get-CheckedFile 'https://npcap.com/dist/npcap-sdk-1.16.zip' 'npcap-sdk.zip' 'F0A8BE7778EE3AE1B99BBBECB27A3FF0F6C111A4093F1C78C5C5A099607184DB'
Expand-Archive $sdk -DestinationPath (Join-Path $directory 'sdk') -Force
$sdkLib = (Resolve-Path (Join-Path $directory 'sdk/Lib/x64')).Path
"LIB=$sdkLib;$env:LIB" >> $env:GITHUB_ENV
"NPCAP_SDK_LIB=$sdkLib" >> $env:GITHUB_ENV

$windivert = Get-CheckedFile 'https://github.com/basil00/WinDivert/releases/download/v2.2.2/WinDivert-2.2.2-A.zip' 'windivert.zip' '63CB41763BB4B20F600B6DE04E991A9C2BE73279E317D4D82F237B150C5F3F15'
Expand-Archive $windivert -DestinationPath (Join-Path $directory 'windivert') -Force
$windivertRoot = (Resolve-Path (Join-Path $directory 'windivert/WinDivert-2.2.2-A')).Path
"WINDIVERT_DRIVER=$windivertRoot\x64\WinDivert64.sys" >> $env:GITHUB_ENV
"WINDIVERT_LICENSE=$windivertRoot\LICENSE" >> $env:GITHUB_ENV

# The free Npcap installer is interactive. Extract the user-mode DLLs solely for
# loader/unit checks on this ephemeral runner. They are never release assets;
# this is not an installation or a test of the Npcap kernel driver.
$installer = Get-CheckedFile 'https://npcap.com/dist/npcap-1.90.exe' 'npcap.exe' '682B57E1C05EF375931975C9D6DED40888F4BFDF9F4B11571E654F962F1199B6'
$extracted = Join-Path $directory 'npcap'
& 'C:/Program Files/7-Zip/7z.exe' x $installer "-o$extracted" '-ir!$SYSDIR\Npcap\Packet.dll' '-ir!$SYSDIR\Npcap\wpcap.dll' -y
if ($LASTEXITCODE -ne 0) { throw 'Could not extract Npcap DLLs for CI checks.' }
$runtime = (Resolve-Path (Join-Path $extracted '$SYSDIR/Npcap')).Path
foreach ($name in @('Packet.dll', 'wpcap.dll')) {
    if (-not (Test-Path (Join-Path $runtime $name))) { throw "Missing CI runtime: $name" }
}
$runtime >> $env:GITHUB_PATH
