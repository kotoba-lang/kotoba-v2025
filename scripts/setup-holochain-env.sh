#!/usr/bin/env bash
set -e

# Holochain環境セットアップスクリプト
# Holochain CLIのインストールとテスト環境のセットアップを行います

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "🌐 Holochain環境セットアップ"
echo "============================"
echo ""

# Holochain CLIの確認
echo "📦 Holochain CLIの確認"
echo "----------------------------------------"

if command -v holochain &> /dev/null; then
    echo "✅ holochain CLIが見つかりました: $(holochain --version 2>&1 || echo 'version不明')"
    HOLOCHAIN_CMD="holochain"
elif command -v hc &> /dev/null; then
    echo "✅ hc (Holochain CLI)が見つかりました: $(hc --version 2>&1 || echo 'version不明')"
    HOLOCHAIN_CMD="hc"
else
    echo "⚠️  Holochain CLIが見つかりません"
    echo ""
    echo "インストール方法:"
    echo ""
    echo "1. Nixを使用する場合（推奨）:"
    echo "   nix-shell https://holochain.org/install"
    echo ""
    echo "2. Cargoを使用する場合:"
    echo "   cargo install holochain-cli"
    echo ""
    echo "3. バイナリをダウンロードする場合:"
    echo "   https://github.com/holochain/holochain/releases"
    echo ""
    echo "詳細: https://developer.holochain.org/docs/install/"
    echo ""
    exit 1
fi

# Holochainバージョンの確認
echo ""
echo "📋 Holochainバージョン情報"
echo "----------------------------------------"
$HOLOCHAIN_CMD --version 2>&1 || echo "バージョン情報の取得に失敗しました"

# DNAパッケージングツールの確認
echo ""
echo "📦 DNAパッケージングツールの確認"
echo "----------------------------------------"

if $HOLOCHAIN_CMD dna --help &> /dev/null; then
    echo "✅ DNAパッケージングツールが利用可能です"
else
    echo "⚠️  DNAパッケージングツールが見つかりません"
    echo "   Holochain CLIのバージョンが古い可能性があります"
fi

# WASMファイルの確認
echo ""
echo "📦 WASMファイルの確認"
echo "----------------------------------------"
WASM_FILE="target/wasm32-unknown-unknown/release/kotobas_tamaki_holochain.wasm"

if [ -f "$WASM_FILE" ]; then
    echo "✅ WASMファイルが見つかりました: $WASM_FILE"
    ls -lh "$WASM_FILE"
else
    echo "⚠️  WASMファイルが見つかりません: $WASM_FILE"
    echo "   まずWASMファイルをビルドしてください: ./scripts/build-wasm.sh"
fi

# DNAファイルの確認
echo ""
echo "📄 DNAファイルの確認"
echo "----------------------------------------"
DNA_FILE="crates/kotobas-tamaki-holochain/dna/kotobasos.dna.yaml"

if [ -f "$DNA_FILE" ]; then
    echo "✅ DNAファイルが見つかりました: $DNA_FILE"
    
    # DNAパッケージの生成を試行
    if [ -f "$WASM_FILE" ]; then
        echo ""
        echo "🔨 DNAパッケージの生成を試行"
        echo "----------------------------------------"
        cd crates/kotobas-tamaki-holochain
        
        if $HOLOCHAIN_CMD dna pack dna/ 2>&1 | tee /tmp/holochain-dna-pack.log; then
            echo "✅ DNAパッケージが正常に生成されました"
            if [ -f "dna/kotobasos.dna" ]; then
                ls -lh "dna/kotobasos.dna"
            fi
        else
            echo "⚠️  DNAパッケージの生成に失敗しました（詳細は /tmp/holochain-dna-pack.log を確認）"
        fi
        
        cd "$PROJECT_ROOT"
    fi
else
    echo "❌ DNAファイルが見つかりません: $DNA_FILE"
fi

# テスト環境のセットアップ情報
echo ""
echo "🧪 テスト環境のセットアップ"
echo "----------------------------------------"
echo ""
echo "統合テストを実行するには、以下のいずれかの方法でHolochainテスト環境をセットアップしてください:"
echo ""
echo "1. Holochain Sandbox (推奨):"
echo "   $HOLOCHAIN_CMD sandbox generate"
echo "   $HOLOCHAIN_CMD sandbox run"
echo ""
echo "2. カスタムテスト環境:"
echo "   Holochainテストフレームワークを使用してテスト環境を構築"
echo ""
echo "詳細: https://developer.holochain.org/docs/testing/"
echo ""

echo "✨ セットアップ完了！"
echo ""

