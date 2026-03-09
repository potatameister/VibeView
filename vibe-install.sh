#!/bin/bash

# VibeView TANK Master Installer
# Designed to be bulletproof even on fresh Termux installs.

set -e

# 1. Colors
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${CYAN}VibeView Tank Installer${NC}"
echo "--------------------------"

# 2. Add 'its-pointless' repo for Android Build Tools (d8)
if ! grep -q "pointless" /data/data/com.termux/files/usr/etc/apt/sources.list.d/* 2>/dev/null; then
    echo -e "${YELLOW}Adding 'its-pointless' repository...${NC}"
    
    # We avoid piping to bash. We do it manually.
    pkg update && pkg install -y gnupg curl
    
    # Manually fetch the key
    curl -sL https://its-pointless.github.io/pointless.gpg | apt-key add -
    
    # Manually add the list
    echo "deb https://its-pointless.github.io/files/24 termux extras" > /data/data/com.termux/files/usr/etc/apt/sources.list.d/pointless.list
    
    pkg update
fi

# 3. Install core tools
echo -e "📦 Installing dependencies..."
pkg install -y rust kotlin gradle inotify-tools binutils-is-llvm git build-tools debianutils

# 4. Handle Source Code
INSTALL_DIR="$HOME/.vibeview-src"
echo -e "📡 Fetching source..."

if [ -d "$INSTALL_DIR" ]; then
    cd "$INSTALL_DIR"
    git fetch origin main
    git reset --hard origin/main
else
    git clone https://github.com/potatameister/VibeView.git "$INSTALL_DIR"
    cd "$INSTALL_DIR"
fi

# 5. Build and Install CLI
echo -e "🔨 Building CLI..."
cd vibe-watch
cargo build --release

# 6. Global Installation
TARGET_BIN="/data/data/com.termux/files/usr/bin/vibe"
echo -e "🚀 Finalizing installation..."
ln -sf "$(pwd)/target/release/vibe-watch" "$TARGET_BIN"

# 7. Verification
echo -e "\n${GREEN}✅ VibeView is now ready!${NC}"
echo -e "Try: ${CYAN}vibe doctor${NC}"
echo ""
vibe doctor
