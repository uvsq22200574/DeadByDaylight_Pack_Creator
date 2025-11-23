#!/bin/bash

REMOVE_TARGET=false
CLEAR_BINARIES=false

# Function to display help
function show_help() {
    echo "Usage: $0 [options]"
    echo
    echo "Options:"
    echo "  -rtemp       Remove the 'target/' folder after building"
    echo "  -clear       Delete previously built binaries and the 'target/' folder, then exit"
    echo "  -help        Show this help message and exit"
    echo
    echo "This script builds the Rust project for Linux and Windows targets,"
    echo "appends the project version to binary names, and places them at the project root."
}

# Parse optional flags
for arg in "$@"; do
    case $arg in
        -rtemp)
            REMOVE_TARGET=true
            ;;
        -clear)
            CLEAR_BINARIES=true
            ;;
        -help|--help)
            show_help
            exit 0
            ;;
        *)
            echo "❌ Unknown option: $arg"
            show_help
            exit 1
            ;;
    esac
done

# Detect project name and version from Cargo.toml
PROJECT_NAME=$(awk -F ' = ' '/^name/ { gsub(/"/, "", $2); print $2; exit }' Cargo.toml)
PROJECT_VERSION=$(awk -F ' = ' '/^version/ { gsub(/"/, "", $2); print $2; exit }' Cargo.toml)

if [ -z "$PROJECT_NAME" ] || [ -z "$PROJECT_VERSION" ]; then
    echo "❌ Failed to detect project name or version from Cargo.toml!"
    exit 1
fi

# Clear binaries and target folder if requested
if [ "$CLEAR_BINARIES" = true ]; then
    echo "🗑 Removing previously built binaries and 'target/' folder..."
    rm -f "${PROJECT_NAME}-${PROJECT_VERSION}-Linux" "${PROJECT_NAME}-${PROJECT_VERSION}-Windows.exe"
    rm -rf target
    echo "✅ Cleared."
    exit 0
fi

echo "📦 Project name: $PROJECT_NAME"
echo "📄 Project version: $PROJECT_VERSION"

# Define targets
TARGETS=(
    "x86_64-unknown-linux-gnu"    # Linux
    "x86_64-pc-windows-gnu"       # Windows
)

# Map long target names to simplified names
declare -A TARGET_NAMES
TARGET_NAMES["x86_64-unknown-linux-gnu"]="Linux"
TARGET_NAMES["x86_64-pc-windows-gnu"]="Windows"

# Ensure required targets are installed
echo "🔄 Installing Rust targets..."
for TARGET in "${TARGETS[@]}"; do
    rustup target add "$TARGET"
done

# Build for each target
for TARGET in "${TARGETS[@]}"; do
    echo "🚀 Building for $TARGET..."
    
    cargo build --release --target "$TARGET"
    
    EXT=""
    if [[ "$TARGET" == *"windows"* ]]; then
        EXT=".exe"
        BIN_PATH="target/$TARGET/release/$PROJECT_NAME$EXT"
    else
        BIN_PATH="target/$TARGET/release/$PROJECT_NAME"
        EXT=".bin"
    fi

    SIMPLE_NAME=${TARGET_NAMES[$TARGET]}

    if [ -f "$BIN_PATH" ]; then
        # Place binary at project root with version and simplified name
        cp "$BIN_PATH" "${PROJECT_NAME}-${PROJECT_VERSION}-${SIMPLE_NAME}$EXT"
    else
        echo "⚠️ Failed to build for $TARGET"
    fi
done

# Remove target folder if requested (independent of -clear)
if [ "$REMOVE_TARGET" = true ]; then
    echo "🗑 Removing target folder..."
    rm -rf target
fi

echo "✅ All binaries are at project root"
