#!/usr/bin/env bash
set -e

# すべてのテストを実行するスクリプト
# Holochain環境の状態に応じて適切なテストを実行します

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "🧪 Kotobasos Holochain テストスイート"
echo "======================================"
echo ""

# Holochain環境の確認
HOLOCHAIN_AVAILABLE=false
if command -v hc &> /dev/null || command -v holochain &> /dev/null; then
    HOLOCHAIN_AVAILABLE=true
    echo "✅ Holochain環境が利用可能です"
else
    echo "⚠️  Holochain環境が利用できません"
    echo "   統合テストとエージェント間通信テストはスキップされます"
fi

echo ""

# フェーズ1: 単体テスト
echo "📦 フェーズ1: 単体テスト"
echo "----------------------------------------"
cd crates/kotobas-tamaki-holochain

if cargo test --lib 2>&1 | tee /tmp/holochain-unit-tests.log; then
    echo "✅ 単体テストが成功しました"
else
    echo "⚠️  単体テストに問題があります（詳細は /tmp/holochain-unit-tests.log を確認）"
fi

echo ""

# フェーズ2: 統合テスト（Holochain環境が必要）
if [ "$HOLOCHAIN_AVAILABLE" = true ]; then
    echo "🌐 フェーズ2: 統合テスト"
    echo "----------------------------------------"
    
    # WASMファイルの確認
    WASM_FILE="../../target/wasm32-unknown-unknown/release/kotobas_tamaki_holochain.wasm"
    if [ ! -f "$WASM_FILE" ]; then
        echo "⚠️  WASMファイルが見つかりません。まずビルドしてください:"
        echo "   ./scripts/build-wasm.sh"
        echo ""
    else
        if cargo test --test integration_tests -- --ignored --nocapture 2>&1 | tee /tmp/holochain-integration-tests.log; then
            echo "✅ 統合テストが成功しました"
        else
            echo "⚠️  統合テストに問題があります（詳細は /tmp/holochain-integration-tests.log を確認）"
        fi
    fi
else
    echo "⏭️  フェーズ2: 統合テスト（スキップ - Holochain環境が必要）"
fi

echo ""

# フェーズ3: エージェント間通信テスト（Holochain環境が必要）
if [ "$HOLOCHAIN_AVAILABLE" = true ]; then
    echo "🤝 フェーズ3: エージェント間通信テスト"
    echo "----------------------------------------"
    
    WASM_FILE="../../target/wasm32-unknown-unknown/release/kotobas_tamaki_holochain.wasm"
    if [ ! -f "$WASM_FILE" ]; then
        echo "⚠️  WASMファイルが見つかりません。まずビルドしてください:"
        echo "   ./scripts/build-wasm.sh"
        echo ""
    else
        if cargo test --test agent_communication_tests -- --ignored --nocapture 2>&1 | tee /tmp/holochain-agent-tests.log; then
            echo "✅ エージェント間通信テストが成功しました"
        else
            echo "⚠️  エージェント間通信テストに問題があります（詳細は /tmp/holochain-agent-tests.log を確認）"
        fi
    fi
else
    echo "⏭️  フェーズ3: エージェント間通信テスト（スキップ - Holochain環境が必要）"
fi

echo ""
echo "📊 テスト結果のまとめ"
echo "========================================"
echo ""
echo "テストログ:"
echo "  - 単体テスト: /tmp/holochain-unit-tests.log"
if [ "$HOLOCHAIN_AVAILABLE" = true ]; then
    echo "  - 統合テスト: /tmp/holochain-integration-tests.log"
    echo "  - エージェント間通信テスト: /tmp/holochain-agent-tests.log"
fi
echo ""
echo "✨ テストスイート完了！"
echo ""

