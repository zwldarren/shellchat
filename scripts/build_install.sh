#!/bin/bash

# Configuration area - modify these variables according to your project
BINARY_NAME="schat"                     # Application name
INSTALL_PATH="$HOME/.local/bin"         # Default installation path
CONFIG_DIR="$HOME/.config/$BINARY_NAME" # Configuration directory

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check Rust installation
check_rust() {
    if ! command_exists cargo; then
        echo -e "${RED}Error: Rust environment not installed${NC}"
        echo -e "Please install Rust first: https://www.rust-lang.org/tools/install"
        exit 1
    fi
}

# Build application
build_app() {
    echo -e "${YELLOW}Building application (cargo build --release)...${NC}"
    cargo build --release

    if [ $? -ne 0 ]; then
        echo -e "${RED}Build failed, please check errors${NC}"
        exit 1
    fi

    echo -e "${GREEN}Build successful!${NC}"
}

# Install application
install_app() {
    check_rust
    build_app

    # Get built binary path
    local binary_path="./target/release/$BINARY_NAME"

    if [ ! -f "$binary_path" ]; then
        echo -e "${RED}Error: Binary file not found $binary_path${NC}"
        echo -e "Please check name configuration in Cargo.toml: name = \"$BINARY_NAME\""
        exit 1
    fi

    # Install binary
    echo -e "${YELLOW}Installing to $INSTALL_PATH...${NC}"

    # Create directory if it doesn't exist
    mkdir -p "$INSTALL_PATH"

    # Install to user directory with normal user permissions
    install -m 755 "$binary_path" "$INSTALL_PATH/$BINARY_NAME"
    install_status=$?

    if [ $install_status -ne 0 ]; then
        echo -e "${RED}Installation failed!${NC}"
        exit 1
    fi

    # Create config directory
    mkdir -p "$CONFIG_DIR"
    echo "Installation time: $(date)" >"$CONFIG_DIR/install.info"
    echo "Installation path: $INSTALL_PATH/$BINARY_NAME" >>"$CONFIG_DIR/install.info"
    echo "Version: $("$INSTALL_PATH/$BINARY_NAME" --version 2>/dev/null || echo unknown)" >>"$CONFIG_DIR/install.info"

    echo -e "${GREEN}Successfully installed $BINARY_NAME to $INSTALL_PATH${NC}"
    echo -e "You can run: $BINARY_NAME --help"
}

# Uninstall application
uninstall_app() {
    # Check if file exists
    if [ -f "$INSTALL_PATH/$BINARY_NAME" ]; then
        rm -f "$INSTALL_PATH/$BINARY_NAME"
        echo -e "${GREEN}Removed binary file: $INSTALL_PATH/$BINARY_NAME${NC}"
    fi

    if [ -d "$CONFIG_DIR" ]; then
        rm -rf "$CONFIG_DIR"
        echo -e "${GREEN}Removed config files: $CONFIG_DIR${NC}"
    fi

    echo -e "${GREEN}Successfully uninstalled $BINARY_NAME${NC}"
}

# Main program
case "$1" in
install)
    install_app
    ;;
uninstall)
    uninstall_app
    ;;
*)
    echo "Local installation test script"
    echo "Usage:"
    echo "  ./install.sh install    Build and install application"
    echo "  ./install.sh uninstall  Uninstall application"
    echo ""
    echo "Notes:"
    echo "  1. Requires Rust toolchain (cargo) to be installed"
    echo "  2. Installation location: $INSTALL_PATH/$BINARY_NAME"
    echo "  3. Config files: $CONFIG_DIR"
    exit 1
    ;;
esac
