#!/bin/bash
set -e

echo "🚀 CAP-BEAM Monorepo Installer"

# 1. Check dependencies
command -v cargo >/dev/null 2>&1 || { echo >&2 "Error: Rust (cargo) is required."; exit 1; }
command -v zig >/dev/null 2>&1 || { echo >&2 "Error: Zig is required."; exit 1; }

ROOT_DIR=$(pwd)

# 2. Build CPC (Compiler)
echo "🦀 Building cpc (Rust)..."
cd "$ROOT_DIR/cpc"
cargo build --release

# 3. Build CAP (Orchestrator)
echo "⚡ Building cap (Zig)..."
cd "$ROOT_DIR/cap"
zig build -Doptimize=ReleaseFast

# 4. Final Setup
echo ""
echo "✅ Installation complete!"
echo ""
echo "Please add the following to your PATH:"
echo "export PATH=\"\$PATH:$ROOT_DIR/cap/bin:$ROOT_DIR/cpc/target/release\""
echo ""
echo "Run 'cap init' in a new folder to start your first CP project!"
