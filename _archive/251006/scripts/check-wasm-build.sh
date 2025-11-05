#!/usr/bin/env bash
set -e

# WASMビルド状態確認スクリプト
# WASMファイルの存在とDNAファイルの設定を確認します

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "🔨 WASMビルド状態確認"
echo "===================="
echo ""

# WASMファイルの確認
WASM_FILE="target/wasm32-unknown-unknown/release/kotobas_tamaki_holochain.wasm"
echo "📦 WASMファイルの確認"
echo "----------------------------------------"

if [ -f "$WASM_FILE" ]; then
    echo "✅ WASMファイルが見つかりました: $WASM_FILE"
    ls -lh "$WASM_FILE"
    echo ""
    echo "ファイルサイズ: $(du -h "$WASM_FILE" | cut -f1)"
    echo "最終更新: $(stat -f "%Sm" "$WASM_FILE" 2>/dev/null || stat -c "%y" "$WASM_FILE" 2>/dev/null || echo '不明')"
else
    echo "⚠️  WASMファイルが見つかりません: $WASM_FILE"
    echo ""
    echo "ビルドを試行するには:"
    echo "  ./scripts/build-wasm.sh"
    echo ""
    echo "注意: ワークスペース全体のビルドにはindexmapバージョン競合の問題があります。"
    echo "     クレート単独でのビルドを試行してください。"
fi

# DNAファイルの確認
DNA_FILE="crates/kotobas-tamaki-holochain/dna/kotobasos.dna.yaml"
echo ""
echo "📄 DNAファイルの確認"
echo "----------------------------------------"

if [ -f "$DNA_FILE" ]; then
    echo "✅ DNAファイルが見つかりました: $DNA_FILE"
    
    # YAML構文チェック
    if command -v yq &> /dev/null; then
        if yq eval '.' "$DNA_FILE" > /dev/null 2>&1; then
            echo "✅ DNAファイルのYAML構文は正しいです"
        else
            echo "⚠️  DNAファイルのYAML構文に問題がある可能性があります"
        fi
    else
        echo "ℹ️  yqがインストールされていないため、YAML構文チェックをスキップします"
    fi
    
    # WASMファイルパスの確認
    WASM_PATH_IN_DNA=$(grep "bundled:" "$DNA_FILE" | head -1 | sed 's/.*bundled: *//' | tr -d '"' || echo "")
    if [ -n "$WASM_PATH_IN_DNA" ]; then
        echo ""
        echo "DNAファイルで指定されているWASMパス: $WASM_PATH_IN_DNA"
        FULL_WASM_PATH="crates/kotobas-tamaki-holochain/$WASM_PATH_IN_DNA"
        if [ -f "$FULL_WASM_PATH" ]; then
            echo "✅ 指定されたWASMファイルが存在します"
        else
            echo "⚠️  指定されたWASMファイルが見つかりません: $FULL_WASM_PATH"
        fi
    fi
else
    echo "❌ DNAファイルが見つかりません: $DNA_FILE"
fi

# DNAパッケージの確認
DNA_PACKAGE="crates/kotobas-tamaki-holochain/dna/kotobasos.dna"
echo ""
echo "📦 DNAパッケージの確認"
echo "----------------------------------------"

if [ -f "$DNA_PACKAGE" ]; then
    echo "✅ DNAパッケージが見つかりました: $DNA_PACKAGE"
    ls -lh "$DNA_PACKAGE"
else
    echo "⚠️  DNAパッケージが見つかりません: $DNA_PACKAGE"
    echo ""
    echo "DNAパッケージを生成するには（Holochain CLIが必要）:"
    echo "  cd crates/kotobas-tamaki-holochain"
    echo "  hc dna pack dna/"
    echo "  または"
    echo "  holochain dna pack dna/"
fi

echo ""
echo "✨ 確認完了！"
echo ""

