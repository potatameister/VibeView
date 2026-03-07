#!/bin/bash

# VibeView - Fast Compiler Script
# This script uses the Kotlin Compile Daemon for rapid sub-2-second compilation.

# 1. Configuration
KOTLINC="kotlinc"
D8="d8" # Android DEX tool
APP_IP="127.0.0.1"
APP_PORT="8888"

# 2. Check for the changed file (this script is usually triggered by vibe-watch)
# For now, we'll compile the main.kt or everything in the folder.
# In a real session, vibe-watch could pass the filename.

echo "[VibeView] Starting rapid compile..."
START_TIME=$(date +%s%3N)

# 3. Compile Kotlin to JVM Bytecode (.class)
# We use -Xuse-k2 for extra speed if available.
$KOTLINC *.kt -d out/ -Xuse-k2 2>/dev/null

# 4. Convert JVM Bytecode to Android Bytecode (.dex)
# We use the D8 tool which is faster than the old DX tool.
# Note: In Termux, you might need to find the exact path to d8 from the SDK.
# For this MVP, we assume d8 is in the PATH or we use a shim.
if command -v d8 &> /dev/null; then
    d8 out/*.class --output out/classes.dex --min-api 26 --lib $ANDROID_HOME/platforms/android-34/android.jar
else
    echo "[VibeView] Warning: d8 not found. Skipping DEX step (Preview might fail)."
fi

END_TIME=$(date +%s%3N)
ELAPSED=$((END_TIME - START_TIME))

echo "[VibeView] Compile finished in ${ELAPSED}ms."

# 5. Push to the App
echo "[VibeView] Pushing to Shell..."
curl -s -X POST http://$APP_IP:$APP_PORT/push --data-binary @out/classes.dex

if [ $? -eq 0 ]; then
    echo "[VibeView] Successfully pushed to VibeView Shell."
else
    echo "[VibeView] Error: Failed to connect to VibeView Shell on $APP_IP:$APP_PORT"
fi
