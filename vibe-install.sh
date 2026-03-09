#!/bin/bash

# VibeView ULTIMATE Master Installer
# Designed to work on a 100% fresh Termux install.

set -e

# 1. Colors
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${CYAN}VibeView Ultimate Installer${NC}"
echo "----------------------------"

# 2. Add 'its-pointless' repo for Android Build Tools (d8)
# We use a more robust way to add the repo since the setup script sometimes fails
if ! grep -q "pointless" /data/data/com.termux/files/usr/etc/apt/sources.list.d/* 2>/dev/null; then
    echo -e "${YELLOW}Adding 'its-pointless' repository for Android tools...${NC}"
    # Install gnupg first to handle the key
    pkg update && pkg install -y gnupg curl
    # Fetch the key and add the repo
    curl -sL https://its-pointless.github.io/pointless.gpg | apt-key add -
    echo "deb https://its-pointless.github.io/files/24 termux extras" > /data/data/com.termux/files/usr/etc/apt/sources.list.d/pointless.list
fi

# 3. Install ALL dependencies
echo -e "📦 Installing core developer tools..."
pkg update && pkg install -y \
    rust \
    kotlin \
    gradle \
    inotify-tools \
    binutils-is-llvm \
    git \
    build-tools \
    debianutils

# 4. Handle Source Code
INSTALL_DIR="$HOME/.vibeview-src"
echo -e "📡 Fetching latest VibeView CLI..."

if [ -d "$INSTALL_DIR" ]; then
    cd "$INSTALL_DIR"
    git fetch origin main
    git reset --hard origin/main
else
    git clone https://github.com/potatameister/VibeView.git "$INSTALL_DIR"
    cd "$INSTALL_DIR"
fi

# 5. Build and Install CLI
echo -e "🔨 Building VibeView CLI..."
cd vibe-watch
cargo build --release

# 6. Global Installation
TARGET_BIN="/data/data/com.termux/files/usr/bin/vibe"
echo -e "🚀 Installing 'vibe' to $TARGET_BIN..."
ln -sf "$(pwd)/target/release/vibe-watch" "$TARGET_BIN"

# 7. Final Handshake
echo -e "\n${GREEN}✅ VibeView is now 100% INSTALLED!${NC}"
echo -e "Run: ${CYAN}vibe doctor${NC} to verify."
echo ""
vibe doctor
