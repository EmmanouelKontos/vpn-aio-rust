#!/bin/bash

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Building VPN Manager..."

# Use Windows cargo from WSL
CARGO_CMD="/mnt/c/Users/FRONTDESK_PC/.cargo/bin/cargo.exe"

# Check if Rust is installed
if [ ! -f "$CARGO_CMD" ]; then
    echo "Error: Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Build in release mode
echo "Building in release mode..."
if ! "$CARGO_CMD" build --release; then
    echo "Build failed!"
    exit 1
fi

# Create release directory
mkdir -p release

# Copy binary (Windows executable)
if [ -f "target/release/vpn-manager.exe" ]; then
    cp "target/release/vpn-manager.exe" "release/"
    echo "Build completed successfully!"
    echo "Binary location: release/vpn-manager.exe"
    echo ""
    echo "To run: ./release/vpn-manager.exe"
    echo "To install: Copy vpn-manager.exe to a directory in your PATH"
elif [ -f "target/release/vpn-manager" ]; then
    cp "target/release/vpn-manager" "release/"
    echo "Build completed successfully!"
    echo "Binary location: release/vpn-manager"
    echo ""
    echo "To run: ./release/vpn-manager"
    echo "To install: Copy vpn-manager to a directory in your PATH"
else
    echo "Failed to find binary! Check Cargo.toml for the correct binary name."
    echo "Looking for: target/release/vpn-manager.exe or target/release/vpn-manager"
    exit 1
fi