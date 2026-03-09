#!/bin/bash

# VibeView Master Installer
# This script ensures dependencies are met, builds the CLI, and installs it.

set -e

# 1. Colors for feedback
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${CYAN}VibeView Master Installer${NC}"
echo "--------------------------"

# 2. Check for dependencies
echo -e "📦 Checking dependencies..."
PKGS="rust kotlin gradle inotify-tools binutils-is-llvm git"
MISSING=""

for pkg in $PKGS; do
    if ! command -v "$pkg" &> /dev/null && [ "$pkg" != "binutils-is-llvm" ]; then
        MISSING="$MISSING $pkg"
    fi
done

if [ -n "$MISSING" ]; then
    echo -e "${YELLOW}Missing packages:${MISSING}. Installing...${NC}"
    pkg update && pkg install -y $MISSING binutils-is-llvm
else
    echo -e "${GREEN}✓ All packages installed.${NC}"
fi

# 3. Handle Project Directory
INSTALL_DIR="$HOME/.vibeview-src"

if [ -d "vibe-watch" ]; then
    # We are likely inside the repo already
    echo -e "${GREEN}✓ Detected local source code.${NC}"
    CLI_DIR="$(pwd)/vibe-watch"
else
    # We need to fetch the source code
    echo -e "📡 Fetching VibeView source code..."
    if [ -d "$INSTALL_DIR" ]; then
        cd "$INSTALL_DIR"
        git pull origin main
    else
        git clone https://github.com/potatameister/VibeView.git "$INSTALL_DIR"
        cd "$INSTALL_DIR"
    fi
    CLI_DIR="$(pwd)/vibe-watch"
fi

# 4. Build the CLI
echo -e "🔨 Building VibeView CLI..."
cd "$CLI_DIR"
cargo build --release

# 5. Install to system path
TARGET_BIN="/data/data/com.termux/files/usr/bin/vibe"
echo -e "🚀 Installing 'vibe' to $TARGET_BIN..."
ln -sf "$CLI_DIR/target/release/vibe-watch" "$TARGET_BIN"

echo -e "\n${GREEN}✅ VibeView CLI installed successfully!${NC}"
echo -e "Try running: ${CYAN}vibe doctor${NC}"
echo ""
vibe doctor
