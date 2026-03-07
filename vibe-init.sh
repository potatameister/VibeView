#!/bin/bash

# VibeView Project Initializer
# This script sets up a "Vibecoding" project that can be instantly previewed in the VibeView app.

PROJECT_NAME=${1:-"my-vibe-project"}

echo "[VibeView] Initializing project: $PROJECT_NAME"

mkdir -p "$PROJECT_NAME/out"
cd "$PROJECT_NAME" || exit

# Create the standard entry point VibeSnippet.kt
cat <<EOF > VibeSnippet.kt
package com.potatameister.vibeview

import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.MaterialTheme

object VibeSnippet {
    @Composable
    fun getContent() {
        Column {
            Text(
                text = "Hello from $PROJECT_NAME!",
                style = MaterialTheme.typography.headlineMedium
            )
            Text(
                text = "Edit VibeSnippet.kt and save to see changes live.",
                style = MaterialTheme.typography.bodyLarge
            )
        }
    }
}
EOF

# Copy the compile script to the project
cp ~/project/VibeView/vibe-compile.sh .
chmod +x vibe-compile.sh

echo "[VibeView] Project initialized."
echo "[VibeView] Usage: vibe-watch . (Run this in a separate terminal)"
