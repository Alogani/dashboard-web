#!/bin/sh

# Exit on error
set -e

# Get the project root directory
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PACKAGE_DIR="$PROJECT_ROOT/package"

echo "Building home-webserver in release mode..."
cd "$PROJECT_ROOT"
cargo build --release --target x86_64-unknown-linux-musl

echo "Copying binary to package directory..."
cp "$PROJECT_ROOT/target/x86_64-unknown-linux-musl/release/home-webserver" "$PACKAGE_DIR/"

echo "Build complete. Binary is available at $PACKAGE_DIR/home-webserver"