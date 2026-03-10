#!/bin/bash

# VibeView ULTIMATE Master Installer
# Designed to be bulletproof even on fresh Termux installs.

set -e

# 1. Colors
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${CYAN}VibeView Ultimate Installer${NC}"
echo "--------------------------"

# 2. Setup Termux Permissions (External Apps)
echo -e "🔐 Configuring Termux permissions..."
mkdir -p ~/.termux
if ! grep -q "allow-external-apps = true" ~/.termux/termux.properties 2>/dev/null; then
    echo "allow-external-apps = true" >> ~/.termux/termux.properties
    termux-reload-settings
    echo -e "${GREEN}✓ External app access enabled.${NC}"
else
    echo -e "${GREEN}✓ External app access already enabled.${NC}"
fi

# 3. Add 'its-pointless' repo for Android Build Tools
if ! grep -q "pointless" /data/data/com.termux/files/usr/etc/apt/sources.list.d/* 2>/dev/null; then
    echo -e "${YELLOW}Adding 'its-pointless' repository...${NC}"
    pkg update && pkg install -y gnupg curl
    curl -sL https://its-pointless.github.io/pointless.gpg | apt-key add -
    echo "deb https://its-pointless.github.io/files/24 termux extras" > /data/data/com.termux/files/usr/etc/apt/sources.list.d/pointless.list
    pkg update
fi

# 4. Install ALL dependencies
echo -e "📦 Installing dependencies..."
pkg install -y rust kotlin gradle inotify-tools binutils-is-llvm git android-tools debianutils dx

# 5. Fix Kotlin PATH
KOTLIN_BIN="/data/data/com.termux/files/usr/opt/kotlin/bin"
if [ -d "$KOTLIN_BIN" ] && ! echo "$PATH" | grep -q "$KOTLIN_BIN"; then
    echo -e "${YELLOW}Fixing Kotlin PATH...${NC}"
    echo "export PATH=\$PATH:$KOTLIN_BIN" >> ~/.bashrc
    export PATH=$PATH:$KOTLIN_BIN
fi

# 6. Handle Source Code
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

# 7. Setup Android Classpath Shim
echo -e "📦 Setting up Android SDK shim..."
SYSTEM_JAR="/system/framework/framework.jar"
SHIM_PATH="$INSTALL_DIR/android-shim.jar"

if [ -f "$SYSTEM_JAR" ]; then
    echo -e "${GREEN}✓ Found system framework. Using as shim.${NC}"
    ln -sf "$SYSTEM_JAR" "$SHIM_PATH"
else
    echo -e "📡 Downloading remote shim..."
    curl -sL "https://github.com/potatameister/VibeView/releases/download/v0.1-alpha/android-shim.jar" -o "$SHIM_PATH" || \
    echo -e "${YELLOW}Warning: Could not provide SDK shim. Compilation might fail.${NC}"
fi

# 8. Build and Install CLI
echo -e "🔨 Building CLI..."
cd vibe-watch
cargo build --release

# 8. Global Installation
TARGET_BIN="/data/data/com.termux/files/usr/bin/vibe"
echo -e "🚀 Finalizing installation..."
ln -sf "$(pwd)/target/release/vibe-watch" "$TARGET_BIN"

# 9. Verification
echo -e "\n${GREEN}✅ VibeView is now 100% READY!${NC}"
echo -e "Run: ${CYAN}source ~/.bashrc${NC} then ${CYAN}vibe doctor${NC}"
echo ""
vibe doctor
