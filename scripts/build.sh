#!/usr/bin/env bash
# build.sh – build automation for Astravon DSL

set -e  # exit on error

echo "🔨 Building Astravon DSL compiler and runtime..."
# Clean and compile with Cargo
cargo clean
cargo build --release

# (Optional) generate docs
echo "📚 Generating documentation..."
cargo doc --no-deps

echo "✅ Build complete. Binaries in ./target/release/"
