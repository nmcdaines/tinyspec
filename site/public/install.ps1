# Install script for tinyspec (Windows)
# Usage: irm https://tinyspec.dev/install.ps1 | iex
$ErrorActionPreference = 'Stop'

$Repo = 'nmcdaines/tinyspec'
$InstallDir = Join-Path $env:LOCALAPPDATA 'tinyspec'

function Main {
    Detect-Platform
    Fetch-LatestVersion
    Download-AndInstall
    Add-ToPath
    Print-Success
}

function Detect-Platform {
    $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
    switch ($arch) {
        'X64' {
            $script:Target = 'x86_64-pc-windows-msvc'
        }
        default {
            Write-Error "  error: Unsupported architecture: $arch. Use 'cargo install tinyspec' instead."
            exit 1
        }
    }
    $script:Artifact = "tinyspec-$Target.zip"
    Write-Host "  info: Detected platform: $Target"
}

function Fetch-LatestVersion {
    Write-Host '  info: Fetching latest release...'
    try {
        $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -Headers @{ 'User-Agent' = 'tinyspec-installer' }
        $script:Version = $release.tag_name
    }
    catch {
        Write-Error '  error: Failed to determine latest version. Check your internet connection.'
        exit 1
    }
    if (-not $Version) {
        Write-Error '  error: Failed to determine latest version.'
        exit 1
    }
    Write-Host "  info: Latest version: $Version"
    $script:BaseUrl = "https://github.com/$Repo/releases/download/$Version"
}

function Download-AndInstall {
    $tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) "tinyspec-install-$([System.Guid]::NewGuid().ToString('N').Substring(0,8))"
    New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null

    try {
        Write-Host "  info: Downloading $Artifact..."
        $zipPath = Join-Path $tmpDir $Artifact
        Invoke-WebRequest -Uri "$BaseUrl/$Artifact" -OutFile $zipPath -UseBasicParsing

        Write-Host '  info: Downloading checksums...'
        $checksumsPath = Join-Path $tmpDir 'checksums.txt'
        try {
            Invoke-WebRequest -Uri "$BaseUrl/checksums.txt" -OutFile $checksumsPath -UseBasicParsing
            Verify-Checksum -ZipPath $zipPath -ChecksumsPath $checksumsPath
        }
        catch {
            Write-Host '  warn: Checksums not available for this release, skipping verification'
        }

        Write-Host '  info: Extracting...'
        Expand-Archive -Path $zipPath -DestinationPath $tmpDir -Force

        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        $exePath = Join-Path $tmpDir 'tinyspec.exe'
        Copy-Item -Path $exePath -Destination (Join-Path $InstallDir 'tinyspec.exe') -Force
    }
    finally {
        Remove-Item -Path $tmpDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

function Verify-Checksum {
    param(
        [string]$ZipPath,
        [string]$ChecksumsPath
    )

    $checksums = Get-Content $ChecksumsPath
    $expectedLine = $checksums | Where-Object { $_ -match $Artifact }
    if (-not $expectedLine) {
        Write-Host "  warn: No checksum found for $Artifact, skipping verification"
        return
    }

    $expected = ($expectedLine -split '\s+')[0]
    $actual = (Get-FileHash -Path $ZipPath -Algorithm SHA256).Hash.ToLower()

    if ($expected -ne $actual) {
        Write-Error @"
  error: Checksum verification failed!
  Expected: $expected
  Actual:   $actual
The downloaded file may be corrupted or tampered with.
"@
        exit 1
    }

    Write-Host '  info: Checksum verified'
}

function Add-ToPath {
    $userPath = [System.Environment]::GetEnvironmentVariable('Path', 'User')
    if ($userPath -split ';' -contains $InstallDir) {
        return
    }

    $newPath = "$userPath;$InstallDir"
    [System.Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    $env:Path = "$env:Path;$InstallDir"
    $script:PathUpdated = $true
}

function Print-Success {
    Write-Host ''
    Write-Host "  tinyspec $Version installed to $InstallDir\tinyspec.exe"
    Write-Host ''
    if ($script:PathUpdated) {
        Write-Host '  info: Added to your PATH. Restart your terminal for the change to take effect.'
        Write-Host ''
    }
}

Main
