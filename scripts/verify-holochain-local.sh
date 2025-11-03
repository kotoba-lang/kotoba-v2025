#!/usr/bin/env bash
set -e

# Kotobasos Holochain ローカル動作検証スクリプト
# 各フェーズの検証を段階的に実行します

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "🔍 Kotobasos Holochain ローカル動作検証"
echo "========================================"
echo ""

# フェーズ1: 環境セットアップとビルド確認
echo "📦 フェーズ1: 環境セットアップとビルド確認"
echo "----------------------------------------"

# WASMターゲットの確認
if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    echo "📦 Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
else
    echo "✅ wasm32-unknown-unknown target is installed"
fi

# フェーズ2: 単体テストの実行
echo ""
echo "🧪 フェーズ2: 単体テストの実行"
echo "----------------------------------------"
cd crates/kotobas-tamaki-holochain

if cargo test --lib 2>&1 | tee /tmp/holochain-unit-tests.log; then
    echo "✅ 単体テストが成功しました"
else
    echo "⚠️  単体テストに問題があります（詳細は /tmp/holochain-unit-tests.log を確認）"
    echo "ワークスペースの依存関係の問題の可能性があります"
fi

# フェーズ3: WASMビルドの試行
echo ""
echo "🔨 フェーズ3: WASMビルドの試行"
echo "----------------------------------------"

if cargo build --target wasm32-unknown-unknown --release 2>&1 | tee /tmp/holochain-wasm-build.log; then
    WASM_FILE="../../target/wasm32-unknown-unknown/release/kotobas_tamaki_holochain.wasm"
    if [ -f "$WASM_FILE" ]; then
        echo "✅ WASMファイルが正常にビルドされました: $WASM_FILE"
        ls -lh "$WASM_FILE"
    else
        echo "⚠️  WASMファイルが見つかりません: $WASM_FILE"
    fi
else
    echo "⚠️  WASMビルドに問題があります（詳細は /tmp/holochain-wasm-build.log を確認）"
    echo "ワークスペースの依存関係の問題の可能性があります"
fi

# DNAファイルの検証
echo ""
echo "📄 DNAファイルの検証"
echo "----------------------------------------"
DNA_FILE="dna/kotobasos.dna.yaml"
if [ -f "$DNA_FILE" ]; then
    echo "✅ DNAファイルが見つかりました: $DNA_FILE"
    # YAML構文チェック（yqがインストールされている場合）
    if command -v yq &> /dev/null; then
        if yq eval '.' "$DNA_FILE" > /dev/null 2>&1; then
            echo "✅ DNAファイルのYAML構文は正しいです"
        else
            echo "⚠️  DNAファイルのYAML構文に問題がある可能性があります"
        fi
    else
        echo "ℹ️  yqがインストールされていないため、YAML構文チェックをスキップします"
    fi
else
    echo "❌ DNAファイルが見つかりません: $DNA_FILE"
fi

# フェーズ4: Holochain環境の確認
echo ""
echo "🌐 フェーズ4: Holochain環境の確認"
echo "----------------------------------------"

if command -v holochain &> /dev/null; then
    echo "✅ holochain CLIがインストールされています: $(holochain --version 2>&1 || echo 'version不明')"
elif command -v hc &> /dev/null; then
    echo "✅ hc (Holochain CLI)がインストールされています: $(hc --version 2>&1 || echo 'version不明')"
else
    echo "⚠️  Holochain CLIが見つかりません"
    echo "   統合テストとエージェント間通信テストにはHolochain環境が必要です"
    echo "   インストール方法: https://developer.holochain.org/docs/install/"
fi

# 検証結果のまとめ
echo ""
echo "📊 検証結果のまとめ"
echo "========================================"
echo ""
echo "検証ログ:"
echo "  - 単体テスト: /tmp/holochain-unit-tests.log"
echo "  - WASMビルド: /tmp/holochain-wasm-build.log"
echo ""
echo "次のステップ:"
echo "  1. ワークスペースの依存関係を修正（必要に応じて）"
echo "  2. Holochain環境をセットアップ（統合テスト用）"
echo "  3. 統合テストを実行: cargo test --test integration_tests -- --ignored"
echo "  4. エージェント間通信テストを実行: cargo test --test agent_communication_tests -- --ignored"
echo ""

