#!/bin/bash
set -e

echo "Building VPN Manager..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Build in release mode
echo "Building in release mode..."
cargo build --release

# Create release directory
mkdir -p release

# Copy binary
cp target/release/vpn-manager release/

echo "Build completed successfully!"
echo "Binary location: release/vpn-manager"
echo ""
echo "To run: ./release/vpn-manager"
echo "To install system-wide: sudo cp release/vpn-manager /usr/local/bin/"