#!/bin/bash

# VibeView CLI Installer
# This script builds the Rust CLI and symlinks it to your local bin directory.

set -e

PROJECT_ROOT=$(pwd)
CLI_DIR="$PROJECT_ROOT/vibe-watch"
BIN_NAME="vibe"
TARGET_BIN="/data/data/com.termux/files/usr/bin/$BIN_NAME"

echo "🔨 Building VibeView CLI..."
cd "$CLI_DIR"
cargo build --release

echo "🚀 Installing '$BIN_NAME' to system path..."
# Use a symlink or copy. Symlink is better for development.
ln -sf "$CLI_DIR/target/release/vibe-watch" "$TARGET_BIN"

echo ""
echo "✅ VibeView CLI installed successfully!"
echo "You can now run '$BIN_NAME' from anywhere."
echo ""
vibe doctor
