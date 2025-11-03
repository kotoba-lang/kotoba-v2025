#!/usr/bin/env bash
set -e

# WASMビルドスクリプト
# Kotobasos Holochain zomeをWASM形式でビルドします

echo "🔨 Building Kotobasos Holochain WASM..."

# プロジェクトルートに移動
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# WASMターゲットがインストールされているか確認
if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    echo "📦 Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# クレートディレクトリに移動
cd crates/kotobas-tamaki-holochain

# リリースビルド
echo "🚀 Building WASM (release mode)..."
cargo build --target wasm32-unknown-unknown --release

# WASMファイルのパス
WASM_FILE="../../target/wasm32-unknown-unknown/release/kotobas_tamaki_holochain.wasm"

# ファイルが存在するか確認
if [ -f "$WASM_FILE" ]; then
    echo "✅ WASM file built successfully: $WASM_FILE"
    ls -lh "$WASM_FILE"
else
    echo "❌ WASM file not found at expected path: $WASM_FILE"
    exit 1
fi

# DNAファイルのパスを確認
DNA_FILE="dna/kotobasos.dna.yaml"
if [ -f "$DNA_FILE" ]; then
    echo "✅ DNA file found: $DNA_FILE"
else
    echo "⚠️  DNA file not found: $DNA_FILE"
fi

echo "✨ Build complete!"

