#!/usr/bin/env bash
set -e

# 統合テスト実行スクリプト
# Holochain環境がセットアップされている場合に統合テストを実行します

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "🧪 Kotobasos Holochain 統合テスト実行"
echo "======================================"
echo ""

# Holochain環境の確認
if ! command -v holochain &> /dev/null && ! command -v hc &> /dev/null; then
    echo "❌ Holochain CLIが見つかりません"
    echo "   統合テストにはHolochain環境が必要です"
    echo "   インストール方法: https://developer.holochain.org/docs/install/"
    exit 1
fi

# WASMファイルの確認
WASM_FILE="target/wasm32-unknown-unknown/release/kotobas_tamaki_holochain.wasm"
if [ ! -f "$WASM_FILE" ]; then
    echo "⚠️  WASMファイルが見つかりません: $WASM_FILE"
    echo "   まずWASMファイルをビルドしてください: ./scripts/build-wasm.sh"
    exit 1
fi

echo "✅ WASMファイルが見つかりました: $WASM_FILE"

# DNAファイルの確認
DNA_FILE="crates/kotobas-tamaki-holochain/dna/kotobasos.dna.yaml"
if [ ! -f "$DNA_FILE" ]; then
    echo "❌ DNAファイルが見つかりません: $DNA_FILE"
    exit 1
fi

echo "✅ DNAファイルが見つかりました: $DNA_FILE"

# 統合テストの実行
echo ""
echo "🚀 統合テストを実行します..."
echo "----------------------------------------"
cd crates/kotobas-tamaki-holochain

# ignoreフラグを外してテストを実行
cargo test --test integration_tests -- --nocapture 2>&1 | tee /tmp/holochain-integration-tests.log

if [ ${PIPESTATUS[0]} -eq 0 ]; then
    echo ""
    echo "✅ 統合テストが成功しました"
else
    echo ""
    echo "⚠️  統合テストに問題があります（詳細は /tmp/holochain-integration-tests.log を確認）"
    exit 1
fi

