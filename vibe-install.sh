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

# 2. Setup Termux Permissions
echo -e "🔐 Configuring Termux permissions..."
mkdir -p ~/.termux
if ! grep -q "allow-external-apps = true" ~/.termux/termux.properties 2>/dev/null; then
    echo "allow-external-apps = true" >> ~/.termux/termux.properties
    termux-reload-settings
fi

# 3. Add 'its-pointless' repo
if ! grep -q "pointless" /data/data/com.termux/files/usr/etc/apt/sources.list.d/* 2>/dev/null; then
    echo -e "${YELLOW}Adding 'its-pointless' repository...${NC}"
    pkg update && pkg install -y gnupg curl
    curl -sL https://its-pointless.github.io/pointless.gpg | apt-key add -
    echo "deb https://its-pointless.github.io/files/24 termux extras" > /data/data/com.termux/files/usr/etc/apt/sources.list.d/pointless.list
    pkg update
fi

# 4. Install ALL dependencies
echo -e "📦 Installing core developer tools..."
pkg install -y rust kotlin gradle inotify-tools binutils-is-llvm git android-tools debianutils dx

# 5. Fix Kotlin PATH
KOTLIN_BIN="/data/data/com.termux/files/usr/opt/kotlin/bin"
if [ -d "$KOTLIN_BIN" ]; then
    echo "export PATH=\$PATH:$KOTLIN_BIN" >> ~/.bashrc
    export PATH=$PATH:$KOTLIN_BIN
fi

# 6. Handle Source Code
INSTALL_DIR="$HOME/.vibeview-src"
echo -e "📡 Fetching VibeView CLI..."

if [ -d "$INSTALL_DIR" ]; then
    cd "$INSTALL_DIR"
    git fetch origin main
    git reset --hard origin/main
else
    git clone https://github.com/potatameister/VibeView.git "$INSTALL_DIR"
    cd "$INSTALL_DIR"
fi

# 7. THE LIBRARY HARVEST (New & Critical)
echo -e "🚜 Harvesting core libraries (AndroidX/Compose)..."
mkdir -p "$INSTALL_DIR/libs"
cd "$INSTALL_DIR"

# Create a tiny gradle project to fetch only the metadata/jars we need
cat <<EOF > fetch-libs.gradle
repositories { google(); mavenCentral() }
configurations { harvest }
dependencies {
    implementation("androidx.compose.ui:ui:1.6.0")
    implementation("androidx.compose.material3:material3:1.2.0")
    implementation("androidx.compose.runtime:runtime:1.6.0")
    implementation("androidx.compose.foundation:foundation:1.6.0")
    implementation("androidx.core:core-ktx:1.12.0")
}
task copyLibs(type: Copy) {
    from configurations.compileClasspath
    into "libs"
}
EOF

# Run gradle to fetch the real JARs (this might take 1-2 mins)
gradle -b fetch-libs.gradle copyLibs --no-daemon

# 8. Build and Install CLI
echo -e "🔨 Building CLI..."
cd vibe-watch
cargo build --release

# 9. Global Installation
TARGET_BIN="/data/data/com.termux/files/usr/bin/vibe"
ln -sf "$(pwd)/target/release/vibe-watch" "$TARGET_BIN"

echo -e "\n${GREEN}✅ VibeView is now 100% READY!${NC}"
echo -e "Try: ${CYAN}vibe doctor${NC}"
echo ""
vibe doctor
