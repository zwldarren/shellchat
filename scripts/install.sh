#!/bin/bash

# Configuration area
BINARY_NAME="schat"
REPO="zwldarren/shellchat"
INSTALL_PATH="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/$BINARY_NAME"

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Detect platform
get_platform() {
    case "$(uname -sm)" in
    "Linux x86_64") echo "x86_64-unknown-linux-musl" ;;
    *) echo "unknown" ;;
    esac
}

install_from_github() {
    local version=${1:-"latest"}
    local platform=$(get_platform)

    if [ "$platform" = "unknown" ]; then
        echo -e "${RED}Unsupported platform: $(uname -sm)${NC}"
        exit 1
    fi

    if [ "$version" = "latest" ]; then
        url="https://github.com/$REPO/releases/latest/download/schat-$platform.tar.gz"
    else
        url="https://github.com/$REPO/releases/download/$version/schat-$platform.tar.gz"
    fi

    local temp_file="/tmp/schat-$platform.tar.gz"

    echo -e "${YELLOW}Downloading $BINARY_NAME from GitHub releases...${NC}"
    if ! curl -v -L "$url" -o "$temp_file" 2>&1 | tee /tmp/curl.log; then
        echo -e "${RED}Failed to download binary${NC}"
        echo -e "${YELLOW}Curl output saved to /tmp/curl.log${NC}"
        exit 1
    fi

    # Verify downloaded file
    if ! file "$temp_file" | grep -q "gzip compressed data"; then
        echo -e "${RED}Downloaded file is not a valid gzip archive${NC}"
        echo -e "${YELLOW}File type: $(file "$temp_file")${NC}"
        exit 1
    fi

    # Create install directory if it doesn't exist
    mkdir -p "$INSTALL_PATH"

    # Extract and install with verbose output
    echo -e "${YELLOW}Extracting archive...${NC}"
    if ! tar -xzvf "$temp_file" -C "$INSTALL_PATH"; then
        echo -e "${RED}Failed to extract archive${NC}"
        exit 1
    fi
    rm "$temp_file"

    chmod +x "$INSTALL_PATH/$BINARY_NAME"

    # Create config directory
    mkdir -p "$CONFIG_DIR"
    echo "Installation time: $(date)" >"$CONFIG_DIR/install.info"
    echo "Installation path: $INSTALL_PATH/$BINARY_NAME" >>"$CONFIG_DIR/install.info"
    echo "Version: $("$INSTALL_PATH/$BINARY_NAME" --version 2>/dev/null || echo unknown)" >>"$CONFIG_DIR/install.info"

    # Add to PATH if needed
    if ! echo ":$PATH:" | grep -q ":$INSTALL_PATH:"; then
        [ -f "$HOME/.bashrc" ] && echo "export PATH=\"\$PATH:$INSTALL_PATH\"" >>"$HOME/.bashrc"
        [ -f "$HOME/.zshrc" ] && echo "export PATH=\"\$PATH:$INSTALL_PATH\"" >>"$HOME/.zshrc"
        echo -e "${GREEN}Added installation directory to PATH${NC}"
        echo -e "${YELLOW}Note: You may need to restart your terminal or run 'source ~/.bashrc'${NC}"
    fi

    echo -e "${GREEN}Successfully installed $BINARY_NAME to $INSTALL_PATH${NC}"
    echo -e "Run with: $BINARY_NAME --help"
}

uninstall_app() {
    # Remove binary file
    if [ -f "$INSTALL_PATH/$BINARY_NAME" ]; then
        rm -f "$INSTALL_PATH/$BINARY_NAME"
        echo -e "${GREEN}Removed binary: $INSTALL_PATH/$BINARY_NAME${NC}"
    fi

    if [ -d "$CONFIG_DIR" ]; then
        rm -rf "$CONFIG_DIR"
        echo -e "${GREEN}Removed config files: $CONFIG_DIR${NC}"
    fi

    remove_from_path() {
        local file=$1
        [ ! -f "$file" ] && return

        if sed --version 2>/dev/null | grep -q GNU; then
            sed -i "\|$INSTALL_PATH|d" "$file"
        else # macOS
            sed -i '' "\|$INSTALL_PATH|d" "$file"
        fi
    }

    # Remove from PATH
    remove_from_path "$HOME/.bashrc"
    remove_from_path "$HOME/.zshrc"
    echo -e "${GREEN}Removed installation directory from PATH${NC}"

    echo -e "${GREEN}Successfully uninstalled $BINARY_NAME${NC}"
}

# Main program
case "$1" in
install)
    shift
    install_from_github "$@"
    ;;
uninstall)
    uninstall_app
    ;;
*)
    echo "schat CLI installation script"
    echo "Usage:"
    echo "  ./install.sh install [version]    # Install (default: latest)"
    echo "  ./install.sh uninstall           # Uninstall"
    echo ""
    echo "Examples:"
    echo "  ./install.sh install             # Install latest version"
    echo "  ./install.sh install v0.1.0      # Install specific version"
    exit 1
    ;;
esac
