$ErrorActionPreference = "Stop"

$Repo = "danielvictorino/terminal-stickers"
$BinName = "terminal-stickers"
$InstallDir = if ($env:TERMINAL_STICKERS_INSTALL_DIR) { $env:TERMINAL_STICKERS_INSTALL_DIR } else { Join-Path $HOME ".local\bin" }
$ShareDir = if ($env:TERMINAL_STICKERS_SHARE_DIR) { $env:TERMINAL_STICKERS_SHARE_DIR } else { Join-Path (Split-Path $InstallDir -Parent) "share\terminal-stickers" }

$Arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
switch ($Arch) {
    "X64" { $Target = "x86_64-pc-windows-msvc" }
    default { throw "Unsupported architecture: $Arch" }
}

$Archive = "$BinName-$Target.zip"
$BaseUrl = "https://github.com/$Repo/releases/latest/download"
$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Force -Path $TempDir | Out-Null

try {
    $ArchivePath = Join-Path $TempDir $Archive
    $ChecksumPath = Join-Path $TempDir "$Archive.sha256"

    Invoke-WebRequest -Uri "$BaseUrl/$Archive" -OutFile $ArchivePath

    $ChecksumDownloaded = $false
    try {
        Invoke-WebRequest -Uri "$BaseUrl/$Archive.sha256" -OutFile $ChecksumPath
        $ChecksumDownloaded = $true
    } catch {
        Write-Warning "Checksum verification skipped: $($_.Exception.Message)"
    }

    if ($ChecksumDownloaded) {
        $Expected = (Get-Content $ChecksumPath -Raw).Split(" ", [System.StringSplitOptions]::RemoveEmptyEntries)[0].Trim()
        $Actual = (Get-FileHash -Algorithm SHA256 $ArchivePath).Hash.ToLowerInvariant()
        if ($Actual -ne $Expected.ToLowerInvariant()) {
            throw "Checksum mismatch for $Archive"
        }
    }

    Expand-Archive -Force -Path $ArchivePath -DestinationPath $TempDir
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Copy-Item -Force -Path (Join-Path $TempDir "$BinName.exe") -Destination (Join-Path $InstallDir "$BinName.exe")

    $PackedPacks = Join-Path $TempDir "packs"
    if (Test-Path $PackedPacks) {
        New-Item -ItemType Directory -Force -Path (Join-Path $ShareDir "packs") | Out-Null
        Copy-Item -Recurse -Force -Path (Join-Path $PackedPacks "*") -Destination (Join-Path $ShareDir "packs")
    }

    Write-Host "installed $BinName to $(Join-Path $InstallDir "$BinName.exe")"
    Write-Host "installed sticker packs to $(Join-Path $ShareDir "packs")"
    if (($env:PATH -split ";") -notcontains $InstallDir) {
        Write-Host "add $InstallDir to PATH if $BinName is not found"
    }
} finally {
    Remove-Item -Recurse -Force $TempDir -ErrorAction SilentlyContinue
}
