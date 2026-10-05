# fetch-ffmpeg.ps1 - Download BtbN GPL builds of ffmpeg/ffprobe and place them as Tauri sidecar binaries.
# Usage: powershell -ExecutionPolicy Bypass -File scripts/fetch-ffmpeg.ps1
# License note: GPL build, no nonfree components (libfdk_aac disabled). See FFMPEG-BUILD-INFO.txt.

$ErrorActionPreference = "Stop"

$BaseUrl = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest"
$ZipName = "ffmpeg-master-latest-win64-gpl.zip"
$BinariesDir = Join-Path $PSScriptRoot "..\src-tauri\binaries"
$TempDir = Join-Path $env:TEMP "ffforge-fetch-$(Get-Date -Format 'yyyyMMddHHmmss')"

New-Item -ItemType Directory -Force -Path $BinariesDir | Out-Null
New-Item -ItemType Directory -Force -Path $TempDir | Out-Null

$ZipPath = Join-Path $TempDir $ZipName
Write-Host "Downloading $BaseUrl/$ZipName ..."
Invoke-WebRequest -Uri "$BaseUrl/$ZipName" -OutFile $ZipPath

Write-Host "Extracting ..."
Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force

$ffmpeg = Get-ChildItem -Path $TempDir -Recurse -Filter "ffmpeg.exe" | Select-Object -First 1
$ffprobe = Get-ChildItem -Path $TempDir -Recurse -Filter "ffprobe.exe" | Select-Object -First 1
if (-not $ffmpeg -or -not $ffprobe) { throw "ffmpeg.exe / ffprobe.exe not found in archive" }

# Verify no nonfree components (GPL compliance red line)
$buildconf = & $ffmpeg.FullName -buildconf 2>&1 | Out-String
if ($buildconf -match "--enable-libfdk-aac") {
    throw "RED LINE VIOLATION: build contains nonfree libfdk_aac. Refusing to distribute."
}

$triple = "x86_64-pc-windows-msvc"
Copy-Item $ffmpeg.FullName   (Join-Path $BinariesDir "ffmpeg-$triple.exe")   -Force
Copy-Item $ffprobe.FullName (Join-Path $BinariesDir "ffprobe-$triple.exe") -Force

# Record build info for GPL compliance
$info = @(
    "FFmpeg Build Information (GPL compliance record)",
    "==============================================",
    "",
    "## Source",
    "URL: $BaseUrl/$ZipName (BtbN FFmpeg-Builds, win64-gpl)",
    "Download date: $(Get-Date -Format 'yyyy-MM-dd')",
    "",
    "## SHA256",
    "ffmpeg:   $((Get-FileHash (Join-Path $BinariesDir \"ffmpeg-$triple.exe\") -Algorithm SHA256).Hash.ToLower())",
    "ffprobe:  $((Get-FileHash (Join-Path $BinariesDir \"ffprobe-$triple.exe\") -Algorithm SHA256).Hash.ToLower())",
    "",
    "## ffmpeg -version",
    (& (Join-Path $BinariesDir "ffmpeg-$triple.exe") -version 2>&1 | Out-String),
    "",
    "## ffmpeg -buildconf",
    (& (Join-Path $BinariesDir "ffmpeg-$triple.exe") -buildconf 2>&1 | Out-String)
) -join "`n"

$info | Out-File -FilePath (Join-Path $BinariesDir "FFMPEG-BUILD-INFO.txt") -Encoding utf8

Remove-Item -Recurse -Force $TempDir
Write-Host "Done. Binaries placed in src-tauri/binaries/ and FFMPEG-BUILD-INFO.txt updated."
