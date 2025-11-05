#!/usr/bin/env bash
set -e

# Holochain CLIをNix経由でインストール/使用するスクリプト
# sudo権限なしでHolochain CLIを使用できます

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "🌐 Holochain CLI (Nix経由)"
echo "============================"
echo ""

# Nixの確認
if ! command -v nix &> /dev/null; then
    echo "❌ Nixがインストールされていません"
    echo "   まずNixをインストールしてください:"
    echo "   curl -L https://nixos.org/nix/install | sh"
    exit 1
fi

echo "✅ Nixがインストールされています: $(nix --version 2>&1 | head -1)"
echo ""

# Holochain CLIの確認（nix run経由）
echo "📦 Holochain CLIの確認"
echo "----------------------------------------"

# hcコマンドを試行
if nix run --refresh "github:holochain/holonix?ref=main-0.5#hc" -- --version 2>&1 | grep -q "hc\|holochain"; then
    echo "✅ Holochain CLIが利用可能です（Nix経由）"
    echo ""
    echo "使用方法:"
    echo "  nix run \"github:holochain/holonix?ref=main-0.5#hc\" -- <command>"
    echo ""
    echo "例:"
    echo "  nix run \"github:holochain/holonix?ref=main-0.5#hc\" -- --version"
    echo "  nix run \"github:holochain/holonix?ref=main-0.5#hc\" -- dna pack dna/"
    echo ""
    
    # エイリアスまたはラッパースクリプトの作成を提案
    echo "💡 便利なエイリアスを作成するには、以下を~/.bashrcまたは~/.zshrcに追加:"
    echo "   alias hc='nix run \"github:holochain/holonix?ref=main-0.5#hc\" --'"
    echo ""
else
    echo "⚠️  Holochain CLIの確認に失敗しました"
    echo "   初回実行時は、依存関係のダウンロードに時間がかかる場合があります"
    echo ""
    echo "手動で確認:"
    echo "  nix run --refresh \"github:holochain/holonix?ref=main-0.5#hc\" -- --version"
    echo ""
fi

# 公式セットアップスクリプトについて
echo "📋 公式セットアップスクリプトについて"
echo "----------------------------------------"
echo "公式セットアップスクリプトはsudo権限が必要です:"
echo "  bash <(curl https://holochain.github.io/holochain/setup.sh)"
echo ""
echo "このスクリプトは以下の処理を行います:"
echo "  - Nixのバイナリキャッシュの設定（全ユーザー向け）"
echo "  - Holochain開発環境のセットアップ"
echo ""
echo "⚠️  注意: sudo権限が必要なため、手動で実行してください。"
echo "   または、上記のNix経由の方法を使用することもできます。"
echo ""

