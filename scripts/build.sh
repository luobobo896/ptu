#!/bin/bash
# Build script for ptu

set -e

echo "Building ptu..."
cargo build --release

echo "Build complete!"
echo "Binary location: target/release/ptu"
