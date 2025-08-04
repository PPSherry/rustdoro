#!/bin/bash

# Build script for Rustdoro - A Terminal Pomodoro Timer

echo "🍅 Building Rustdoro..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Cargo/Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Check code formatting
echo "🔍 Checking code formatting..."
cargo fmt --check
if [ $? -ne 0 ]; then
    echo "⚠️  Code formatting issues found. Running cargo fmt..."
    cargo fmt
fi

# Run Clippy linter
echo "🔧 Running Clippy linter..."
cargo clippy -- -D warnings
if [ $? -ne 0 ]; then
    echo "⚠️  Clippy warnings found. Please fix them before release."
fi

# Run tests
echo "🧪 Running tests..."
cargo test
if [ $? -ne 0 ]; then
    echo "❌ Tests failed!"
    exit 1
fi

# Build release version
echo "🚀 Building release version..."
cargo build --release
if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✅ Build completed successfully!"
echo ""
echo "📦 Binary location: target/release/rustdoro"
echo "🏃 To run: cargo run --release"
echo "📖 For help: cargo run --release -- --help"
echo ""
echo "🍅 Rustdoro is ready to help you stay productive!" 