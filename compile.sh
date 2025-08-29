#!/bin/bash

# Detect project name from Cargo.toml
PROJECT_NAME=$(awk -F ' = ' '/^name/ { gsub(/"/, "", $2); print $2; exit }' Cargo.toml)

# Check if the project name was detected
if [ -z "$PROJECT_NAME" ]; then
    echo "❌ Failed to detect project name from Cargo.toml!"
    exit 1
fi

echo "📦 Project name: $PROJECT_NAME"

# Define output directory
OUTPUT_DIR="binaries"
mkdir -p "$OUTPUT_DIR"

# Define targets
TARGETS=(
    "x86_64-unknown-linux-gnu"    # Linux
    "x86_64-pc-windows-gnu"       # Windows (requires MinGW-w64)
)

# Ensure required targets are installed
echo "🔄 Installing Rust targets..."
for TARGET in "${TARGETS[@]}"; do
    rustup target add "$TARGET"
done

# Build for each target
for TARGET in "${TARGETS[@]}"; do
    echo "🚀 Building for $TARGET..."
    
    # Build with optimization
    cargo build --release --target "$TARGET"
    
    # Determine output file extension
    EXT=""
    if [[ "$TARGET" == *"windows"* ]]; then
        EXT=".exe"
    fi
    
    # Move the binary to the output directory
    BIN_PATH="target/$TARGET/release/$PROJECT_NAME$EXT"
    if [ -f "$BIN_PATH" ]; then
        mv "$BIN_PATH" "$OUTPUT_DIR/${PROJECT_NAME}-${TARGET}$EXT"
    else
        echo "⚠️ Failed to build for $TARGET"
    fi
done

# Strip binaries (optional, reduces size)
echo "🛠 Stripping binaries..."
for BIN in "$OUTPUT_DIR"/*; do
    if [[ "$BIN" == *.exe ]]; then
        x86_64-w64-mingw32-strip "$BIN"
    else
        strip "$BIN"
    fi
done

echo "✅ All binaries are in '$OUTPUT_DIR/'"
