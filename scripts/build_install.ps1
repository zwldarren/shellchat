<#
.SYNOPSIS
Local installation/uninstallation script for Rust projects

.DESCRIPTION
This script provides the following functionality:
1. Build Rust project (cargo build --release)
2. Install binary to user directory
3. Create config directory
4. Uninstall application

.USAGE
Run in PowerShell:
    .\install.ps1 install     # Install application
    .\install.ps1 uninstall   # Uninstall application

.NOTES
Requires Rust toolchain (cargo) to be installed first
#>

# Configuration section - modify these variables according to your project
$BINARY_NAME = "schat"               # Application name (without .exe extension)
$INSTALL_DIR = "$env:USERPROFILE\.local\bin"    # Installation directory
$CONFIG_DIR = "$env:USERPROFILE\.config\$BINARY_NAME" # Config directory

# Console color definitions
$RED = "Red"
$GREEN = "Green"
$YELLOW = "Yellow"

# Check if Rust is installed
function Check-Rust {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Host "Error: Rust environment not installed" -ForegroundColor $RED
        Write-Host "Please install Rust first: https://www.rust-lang.org/tools/install"
        exit 1
    }
}

# Build application
function Build-App {
    Write-Host "Building application (cargo build --release)..." -ForegroundColor $YELLOW
    cargo build --release
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build failed, please check errors" -ForegroundColor $RED
        exit 1
    }
    
    Write-Host "Build successful!" -ForegroundColor $GREEN
}

# Install application
function Install-App {
    Check-Rust
    Build-App
    
    # Get binary path
    $binaryPath = ".\target\release\$BINARY_NAME.exe"
    
    if (-not (Test-Path $binaryPath)) {
        Write-Host "Error: Cannot find binary file $binaryPath" -ForegroundColor $RED
        Write-Host "Please check name configuration in Cargo.toml: name = `"$BINARY_NAME`""
        exit 1
    }
    
    # Create installation directory
    if (-not (Test-Path $INSTALL_DIR)) {
        New-Item -ItemType Directory -Path $INSTALL_DIR | Out-Null
    }
    
    # Copy file
    Write-Host "Installing to $INSTALL_DIR..." -ForegroundColor $YELLOW
    Copy-Item -Path $binaryPath -Destination "$INSTALL_DIR\$BINARY_NAME.exe" -Force
    
    # Create config directory
    if (-not (Test-Path $CONFIG_DIR)) {
        New-Item -ItemType Directory -Path $CONFIG_DIR | Out-Null
    }
    
    # Write installation info
    $installInfo = @"
Installation time: $(Get-Date)
Installation path: $INSTALL_DIR\$BINARY_NAME.exe
Version: $(& "$INSTALL_DIR\$BINARY_NAME.exe" --version 2>$null)
"@
    Set-Content -Path "$CONFIG_DIR\install.info" -Value $installInfo
    
    # Add to PATH (optional)
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if (-not ($currentPath -split ";" -contains $INSTALL_DIR)) {
        $newPath = $currentPath + ";" + $INSTALL_DIR
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        Write-Host "Added installation directory to user PATH environment variable" -ForegroundColor $GREEN
        Write-Host "Note: You need to restart terminal or log out for PATH changes to take effect" -ForegroundColor $YELLOW
    }
    
    Write-Host "Successfully installed $BINARY_NAME to $INSTALL_DIR" -ForegroundColor $GREEN
    Write-Host "You can run: $BINARY_NAME --help"
}

# Uninstall application
function Uninstall-App {
    # Remove binary file
    $binaryPath = "$INSTALL_DIR\$BINARY_NAME.exe"
    if (Test-Path $binaryPath) {
        Remove-Item -Path $binaryPath -Force
        Write-Host "Removed binary file: $binaryPath" -ForegroundColor $GREEN
    }
    
    # Remove config directory
    if (Test-Path $CONFIG_DIR) {
        Remove-Item -Path $CONFIG_DIR -Recurse -Force
        Write-Host "Removed config files: $CONFIG_DIR" -ForegroundColor $GREEN
    }
    
    # Remove from PATH (optional)
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $newPath = ($currentPath -split ";" | Where-Object { $_ -ne $INSTALL_DIR }) -join ";"
    if ($newPath -ne $currentPath) {
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        Write-Host "Removed installation directory from PATH environment variable" -ForegroundColor $GREEN
    }
    
    Write-Host "Successfully uninstalled $BINARY_NAME" -ForegroundColor $GREEN
}

# Main program
if ($args.Count -eq 0) {
    Write-Host "Local installation test script"
    Write-Host "Usage:"
    Write-Host "  .\install.ps1 install     # Build and install application"
    Write-Host "  .\install.ps1 uninstall   # Uninstall application"
    Write-Host ""
    Write-Host "Notes:"
    Write-Host "  1. Requires Rust toolchain (cargo) to be installed"
    Write-Host "  2. Installation location: $INSTALL_DIR\$BINARY_NAME.exe"
    Write-Host "  3. Config files: $CONFIG_DIR"
    exit 1
}

switch ($args[0]) {
    "install" {
        Install-App
    }
    "uninstall" {
        Uninstall-App
    }
    default {
        Write-Host "Invalid argument: $($args[0])" -ForegroundColor $RED
        Write-Host "Valid arguments: install, uninstall"
        exit 1
    }
}