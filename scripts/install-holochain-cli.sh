#!/usr/bin/env bash
set -e

# Holochain CLIインストールスクリプト
# 複数のインストール方法を試行します

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "📦 Holochain CLIインストール"
echo "============================"
echo ""

# 既にインストールされているか確認
if command -v holochain &> /dev/null; then
    echo "✅ holochain CLIは既にインストールされています: $(holochain --version 2>&1 || echo 'version不明')"
    exit 0
elif command -v hc &> /dev/null; then
    echo "✅ hc (Holochain CLI)は既にインストールされています: $(hc --version 2>&1 || echo 'version不明')"
    exit 0
fi

echo "Holochain CLIが見つかりません。インストールを試行します..."
echo ""

# 方法1: Nixを使用（推奨）
if command -v nix &> /dev/null; then
    echo "📦 方法1: Nixを使用したインストール"
    echo "----------------------------------------"
    echo "Nixがインストールされています。"
    echo ""
    echo "以下のコマンドを実行してください（sudo権限が必要な場合があります）:"
    echo ""
    echo "方法1: Holochain公式セットアップスクリプト（推奨）"
    echo "  bash <(curl https://holochain.github.io/holochain/setup.sh)"
    echo ""
    echo "方法2: Holonix Flakeを直接使用（Nix環境がある場合）"
    echo "  nix run --refresh \"github:holochain/holonix?ref=main-0.5#hc-scaffold\" -- --version"
    echo ""
    echo "インストール確認:"
    echo "  新しいターミナルセッションで以下を実行:"
    echo "    hc --version"
    echo "    または"
    echo "    holochain --version"
    echo ""
fi

# 方法2: Cargoを使用
echo "📦 方法2: Cargoを使用したインストール"
echo "----------------------------------------"
echo "Holochain CLIはcrates.ioで利用可能です:"
echo ""
echo "以下のコマンドを実行してください:"
echo "  cargo install holochain_cli"
echo ""
echo "または、GitHubリポジトリからビルドする場合:"
echo "  git clone https://github.com/holochain/holochain.git"
echo "  cd holochain"
echo "  cargo install --path crates/holochain_cli"
echo ""

# 方法3: バイナリをダウンロード
echo "📦 方法3: バイナリをダウンロード"
echo "----------------------------------------"
echo "最新のリリースからバイナリをダウンロード:"
echo "  https://github.com/holochain/holochain/releases"
echo ""
echo "ダウンロード後、バイナリをPATHに追加してください。"
echo ""

echo "詳細なインストール手順は以下を参照してください:"
echo "  https://developer.holochain.org/docs/install/"
echo ""
echo "⚠️  注意: Holochain CLIのインストールには時間がかかる場合があります。"
echo "   インストール後、新しいターミナルセッションを開いてください。"
echo ""

