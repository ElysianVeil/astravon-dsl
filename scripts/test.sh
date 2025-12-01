#!/usr/bin/env bash
# test.sh – run all tests for Astravon DSL

set -e

echo "🧪 Running Astravon DSL test suite..."
cargo test -- --nocapture

# Optionally run integration tests in examples
echo "🔍 Running DSL examples..."
for f in examples/*.avon; do
  echo "Running $f..."
  ./target/debug/astravon "$f" || exit 1
done

echo "✅ All tests passed!"
