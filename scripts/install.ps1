<#
.SYNOPSIS
Installation script for schat CLI tool from GitHub releases

.DESCRIPTION
This script provides the following functionality:
1. Detect system architecture
2. Download appropriate pre-built binary from GitHub releases
3. Install to user directory
4. Create config directory
5. Uninstall application

.USAGE
Run in PowerShell:
    .\install.ps1 install                 # Install latest version
    .\install.ps1 uninstall               # Uninstall application
    .\install.ps1 install -Version v0.1.0 # Install specific version
#>

param(
    [Parameter(Position=0)]
    [string]$Command,
    
    [Parameter(Position=1)]
    [string]$Version = "latest"
)

# Configuration section
$BINARY_NAME = "schat"
$REPO = "zwldarren/shellchat"
$INSTALL_DIR = "$env:USERPROFILE\.local\bin"
$CONFIG_DIR = "$env:USERPROFILE\.config\$BINARY_NAME"

# Console colors
$RED = "Red"
$GREEN = "Green"
$YELLOW = "Yellow"

function Get-Platform {
    if ([Environment]::Is64BitOperatingSystem) {
        return "x86_64-pc-windows-gnu"
    }
    return "unknown"
}

function Install-From-GitHub {
    param([string]$Version)
    
    $platform = Get-Platform
    if ($platform -eq "unknown") {
        Write-Host "Unsupported platform: $env:PROCESSOR_ARCHITECTURE" -ForegroundColor $RED
        exit 1
    }

    if ($Version -eq "latest") {
        $url = "https://github.com/$REPO/releases/latest/download/schat-$platform.tar.gz"
    } else {
        $url = "https://github.com/$REPO/releases/download/$Version/schat-$platform.tar.gz"
    }

    $tempFile = "$env:TEMP\schat-$platform.tar.gz"

    Write-Host "Downloading $BINARY_NAME from GitHub releases..." -ForegroundColor $YELLOW
    Write-Host "Download URL: $url" -ForegroundColor $YELLOW
    try {
        Invoke-WebRequest -Uri $url -OutFile $tempFile -UseBasicParsing -Verbose *>&1 | Out-File "$env:TEMP\install.log"
    } catch {
        Write-Host "Failed to download binary: $_" -ForegroundColor $RED
        Write-Host "Detailed log saved to $env:TEMP\install.log" -ForegroundColor $YELLOW
        exit 1
    }

    # Verify downloaded file
    $fileInfo = Get-Item $tempFile
    Write-Host "Downloaded file size: $($fileInfo.Length) bytes" -ForegroundColor $YELLOW
    if ($fileInfo.Length -lt 100KB) {
        Write-Host "Warning: Downloaded file seems unusually small" -ForegroundColor $YELLOW
    }

    # Extract and install
    if (-not (Test-Path $INSTALL_DIR)) {
        New-Item -ItemType Directory -Path $INSTALL_DIR -Force | Out-Null
    }

    Write-Host "Extracting archive..." -ForegroundColor $YELLOW
    try {
        tar -xzvf $tempFile -C $INSTALL_DIR
        if ($LASTEXITCODE -ne 0) {
            throw "tar exited with code $LASTEXITCODE"
        }
    } catch {
        Write-Host "Failed to extract files: $_" -ForegroundColor $RED
        Write-Host "File type: $(file $tempFile)" -ForegroundColor $YELLOW
        exit 1
    } finally {
        Remove-Item $tempFile -ErrorAction SilentlyContinue
    }

    $exePath = "$INSTALL_DIR\$BINARY_NAME.exe"
    if (Test-Path $exePath) {
        Unblock-File $exePath -ErrorAction SilentlyContinue
    }

    if (-not (Test-Path $CONFIG_DIR)) {
        New-Item -ItemType Directory -Path $CONFIG_DIR -Force | Out-Null
    }

    "Installation time: $(Get-Date)" | Out-File "$CONFIG_DIR\install.info"
    "Installation path: $exePath" | Out-File "$CONFIG_DIR\install.info" -Append
    "Version: $($Version)" | Out-File "$CONFIG_DIR\install.info" -Append

    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if (-not ($currentPath -split ";" -contains $INSTALL_DIR)) {
        $newPath = $currentPath + ";" + $INSTALL_DIR
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        Write-Host "Added installation directory to user PATH" -ForegroundColor $GREEN
        Write-Host "Note: You may need to restart your terminal for changes to take effect" -ForegroundColor $YELLOW
    }

    Write-Host "Successfully installed $BINARY_NAME to $INSTALL_DIR" -ForegroundColor $GREEN
    Write-Host "You can run: $BINARY_NAME --help"
}

function Uninstall-App {
    # 移除二进制文件
    $binaryPath = "$INSTALL_DIR\$BINARY_NAME.exe"
    if (Test-Path $binaryPath) {
        Remove-Item -Path $binaryPath -Force -ErrorAction SilentlyContinue
        Write-Host "Removed binary file: $binaryPath" -ForegroundColor $GREEN
    }

    # Remove config directory
    if (Test-Path $CONFIG_DIR) {
        Remove-Item -Path $CONFIG_DIR -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "Removed config files: $CONFIG_DIR" -ForegroundColor $GREEN
    }

    # Remove from PATH if present
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $newPath = ($currentPath -split ";" | Where-Object { $_ -ne $INSTALL_DIR }) -join ";"
    if ($newPath -ne $currentPath) {
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        Write-Host "Removed installation directory from PATH" -ForegroundColor $GREEN
    }

    Write-Host "Successfully uninstalled $BINARY_NAME" -ForegroundColor $GREEN
}

if ($Command -eq $null) {
    Write-Host "schat CLI installation script"
    Write-Host "Usage:"
    Write-Host "  .\install.ps1 install                 # Install latest version"
    Write-Host "  .\install.ps1 uninstall               # Uninstall application"
    Write-Host "  .\install.ps1 install -Version v0.1.0 # Install specific version"
    exit 1
}

switch ($Command.ToLower()) {
    "install" {
        Install-From-GitHub -Version $Version
    }
    "uninstall" {
        Uninstall-App
    }
    default {
        Write-Host "Invalid command: $Command" -ForegroundColor $RED
        Write-Host "Valid commands: install, uninstall"
        exit 1
    }
}
