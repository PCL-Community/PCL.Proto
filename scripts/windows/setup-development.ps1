param([switch]$CheckOnly)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = (Resolve-Path "$PSScriptRoot/../..").Path
$toolsDir = Join-Path $root 'target/build-tools'
$protocDir = Join-Path $toolsDir 'protoc-33.0'
$windivertDir = Join-Path $toolsDir 'windivert-2.2.2'

function Install-Archive($Url, $Hash, $Destination) {
    New-Item -ItemType Directory -Path $Destination -Force | Out-Null
    $archive = "$Destination.zip"
    Invoke-WebRequest -UseBasicParsing -Uri $Url -OutFile $archive
    if ((Get-FileHash $archive -Algorithm SHA256).Hash -ne $Hash) { throw "Checksum mismatch: $Url" }
    Expand-Archive -LiteralPath $archive -DestinationPath $Destination -Force
}

$protoc = $env:PROTOC
if (-not $protoc) {
    $command = Get-Command protoc -ErrorAction SilentlyContinue
    if ($command) { $protoc = $command.Source }
}
if (-not $protoc -or -not (Test-Path $protoc)) {
    $protoc = Join-Path $protocDir 'bin/protoc.exe'
    if (-not (Test-Path $protoc)) {
        if ($CheckOnly) { throw 'Protobuf compiler is missing. Run bun run setup:windows.' }
        Install-Archive 'https://github.com/protocolbuffers/protobuf/releases/download/v33.0/protoc-33.0-win64.zip' '3742CD49C8B6BD78B6760540367EB0FF62FA70A1032E15DAFE131BFAF296986A' $protocDir
    }
}

$env:PROTOC = $protoc
if (-not (Test-Path (Join-Path $windivertDir 'WinDivert-2.2.2-A/x64/WinDivert64.sys'))) {
    if ($CheckOnly) { throw 'WinDivert driver is missing. Run bun run setup:windows.' }
    Install-Archive 'https://github.com/basil00/WinDivert/releases/download/v2.2.2/WinDivert-2.2.2-A.zip' '63CB41763BB4B20F600B6DE04E991A9C2BE73279E317D4D82F237B150C5F3F15' $windivertDir
}
Write-Host "Protobuf compiler: $protoc"
Write-Host "WinDivert driver: $(Join-Path $windivertDir 'WinDivert-2.2.2-A/x64/WinDivert64.sys')"
Write-Host 'Npcap is installed on demand by the launcher; it is not a build dependency.'
