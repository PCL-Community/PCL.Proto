param(
    [switch]$CheckOnly,
    [string]$DownloadDirectory = (Join-Path $PSScriptRoot 'prerequisites')
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

# The launcher is portable; these are system prerequisites, not an app installer.
# Npcap cannot be bundled without redistribution permission. Download it on the
# user's machine and keep its normal interactive license/driver installation.
$npcapVersion = '1.90'
$npcapHash = '682B57E1C05EF375931975C9D6DED40888F4BFDF9F4B11571E654F962F1199B6'

function Test-Npcap {
    return (Test-Path "$env:SystemRoot/System32/Packet.dll") -and
        (Test-Path 'HKLM:/SYSTEM/CurrentControlSet/Services/npcap') -and
        (Test-Path "$env:SystemRoot/System32/drivers/npcap.sys")
}

function Test-WebView2 {
    foreach ($root in @(
        'HKLM:/SOFTWARE/Microsoft/EdgeUpdate/Clients',
        'HKLM:/SOFTWARE/WOW6432Node/Microsoft/EdgeUpdate/Clients',
        'HKCU:/SOFTWARE/Microsoft/EdgeUpdate/Clients'
    )) {
        foreach ($key in @(Get-ChildItem $root -ErrorAction SilentlyContinue)) {
            $client = Get-ItemProperty $key.PSPath
            if ($client.PSObject.Properties['name'] -and
                $client.PSObject.Properties['pv'] -and
                $client.name -like '*WebView2*' -and
                [version]$client.pv -gt [version]'0.0.0.0') { return $true }
        }
    }
    return $false
}

function Get-VerifiedInstaller($Url, $Path, $Publisher, $Hash) {
    Invoke-WebRequest -UseBasicParsing -Uri $Url -OutFile $Path
    if ($Hash -and (Get-FileHash $Path -Algorithm SHA256).Hash -ne $Hash) {
        throw "Installer checksum mismatch: $Path"
    }
    $signature = Get-AuthenticodeSignature $Path
    if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch $Publisher) {
        throw "Invalid installer signature: $Path"
    }
}

$missing = @()
if (-not (Test-Npcap)) { $missing += 'Npcap (including WinPcap API-compatible mode)' }
if (-not (Test-WebView2)) { $missing += 'Microsoft Edge WebView2 Runtime' }
if ($missing.Count -eq 0) {
    Write-Host 'Windows runtime prerequisites are installed.'
    exit 0
}
if ($CheckOnly) {
    throw "Missing prerequisites: $($missing -join ', '). Run bun run setup:windows or Install-Dependencies.cmd first."
}

$downloadDir = $DownloadDirectory
New-Item -ItemType Directory -Path $downloadDir -Force | Out-Null
if (-not (Test-Npcap)) {
    $installer = Join-Path $downloadDir "npcap-$npcapVersion.exe"
    Get-VerifiedInstaller "https://npcap.com/dist/npcap-$npcapVersion.exe" $installer 'O=Nmap Software LLC(?:,|$)' $npcapHash
    $installer = (Resolve-Path -LiteralPath $installer).Path
    Write-Host 'Complete the Npcap installer with WinPcap API-compatible mode enabled.'
    try {
        $process = Start-Process -FilePath $installer -ArgumentList '/winpcap_mode=yes' -Verb RunAs -WindowStyle Normal -Wait -PassThru
    } catch {
        throw "Could not open the Npcap installer. Run $installer manually with WinPcap API-compatible mode enabled, then rerun this script. $($_.Exception.Message)"
    }
    if ($process.ExitCode -eq 3010) { throw 'Npcap requires a Windows restart. Restart and run this script again.' }
    if ($process.ExitCode -ne 0) { throw "Npcap installer exited with code $($process.ExitCode)." }
    if (-not (Test-Npcap)) { throw 'Npcap installation is incomplete. Enable WinPcap API-compatible mode and retry.' }
}
if (-not (Test-WebView2)) {
    $installer = Join-Path $downloadDir 'MicrosoftEdgeWebview2Setup.exe'
    Get-VerifiedInstaller 'https://go.microsoft.com/fwlink/p/?LinkId=2124703' $installer 'O=Microsoft Corporation(?:,|$)' $null
    $installer = (Resolve-Path -LiteralPath $installer).Path
    $process = Start-Process -FilePath $installer -ArgumentList '/install' -WindowStyle Normal -Wait -PassThru
    if ($process.ExitCode -ne 0 -or -not (Test-WebView2)) { throw 'WebView2 Runtime installation did not complete.' }
}
Write-Host 'Windows runtime prerequisites are installed. You can run PCL.Proto.exe directly.'
