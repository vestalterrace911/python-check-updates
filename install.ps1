# install.ps1 - pycu installer for Windows
#
# Usage:
#   irm https://raw.githubusercontent.com/Logic-py/python-check-updates/main/install.ps1 | iex
#
# Override the install directory:
#   $env:PYCU_INSTALL_DIR = "C:\Tools"; irm ... | iex

$ErrorActionPreference = "Stop"

$Repo   = "Logic-py/python-check-updates"
$Binary = "pycu"
$Target = "x86_64-pc-windows-msvc"

$InstallDir = if ($env:PYCU_INSTALL_DIR) {
    $env:PYCU_INSTALL_DIR
} else {
    Join-Path $env:LOCALAPPDATA "Programs\$Binary"
}

# -- Helpers -------------------------------------------------------------------
function Say  ($msg) { Write-Host $msg -ForegroundColor White }
function Info ($msg) { Write-Host "  * $msg" -ForegroundColor Cyan }
function Err  ($msg) { Write-Error "error: $msg"; exit 1 }

# -- Resolve latest version ----------------------------------------------------
Say "Fetching latest $Binary release..."

try {
    $Release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
} catch {
    Err "could not reach GitHub API - is the repository public? ($_)"
}

$Version = $Release.tag_name
if (-not $Version) { Err "could not determine latest version" }

Info "version : $Version"
Info "platform: windows-x86_64 ($Target)"
Info "install : $InstallDir"

# -- Download ------------------------------------------------------------------
$Archive = "$Binary-$Target.zip"
$Url     = "https://github.com/$Repo/releases/download/$Version/$Archive"

$Asset = $Release.assets | Where-Object { $_.name -eq $Archive }
if (-not $Asset) { Err "no asset '$Archive' found in release $Version" }

Say "Downloading $Archive..."

$Tmp = Join-Path $env:TEMP ([System.IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Path $Tmp | Out-Null

try {
    $ZipPath = Join-Path $Tmp $Archive
    Invoke-WebRequest -Uri $Url -OutFile $ZipPath -UseBasicParsing

    # -- Extract ---------------------------------------------------------------
    Expand-Archive -Path $ZipPath -DestinationPath $Tmp

    $ExePath = Join-Path $Tmp "$Binary.exe"
    if (-not (Test-Path $ExePath)) {
        Err "binary not found in archive - please file a bug at https://github.com/$Repo/issues"
    }

    # -- Install ---------------------------------------------------------------
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    $Dest = Join-Path $InstallDir "$Binary.exe"
    Move-Item -Force $ExePath $Dest

    Say "$Binary $Version installed to $Dest"

    # -- PATH hint -------------------------------------------------------------
    $UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($UserPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("PATH", "$UserPath;$InstallDir", "User")
        Write-Host ""
        Write-Host "hint: $InstallDir was added to your user PATH." -ForegroundColor Yellow
        Write-Host "      Restart your terminal for it to take effect."
        Write-Host ""
    }
} finally {
    Remove-Item -Recurse -Force $Tmp -ErrorAction SilentlyContinue
}
