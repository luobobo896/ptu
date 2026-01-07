#!/bin/bash
# Test script for ptu

set -e

echo "Running tests..."
cargo test --all-features

echo "Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

echo "Checking formatting..."
cargo fmt --check

echo "All checks passed!"
