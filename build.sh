#!/bin/bash
set -e

echo "[hyprspace] Cleaning previous build..."
cargo clean

echo "[hyprspace] Building optimized release binary..."
cargo build --release

echo "[hyprspace] Stripping binary to reduce size..."
strip target/release/hyprspace || true

mkdir -p dist

echo "[hyprspace] Copying binary to dist/ ..."
cp target/release/hyprspace dist/hyprspace

echo ""
echo "========================================"
echo "  Build complete!"
echo "  Binary available at: dist/hyprspace"
echo "========================================"