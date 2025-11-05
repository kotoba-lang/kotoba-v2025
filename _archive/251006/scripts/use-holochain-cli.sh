#!/usr/bin/env bash
# Holochain CLIをNix経由で使用するラッパースクリプト
# このスクリプトを使用すると、nix runコマンドを簡単に実行できます

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Holochain CLIが直接インストールされているか確認
if command -v hc &> /dev/null; then
    exec hc "$@"
elif command -v holochain &> /dev/null; then
    exec holochain "$@"
fi

# Nix経由で実行
exec nix run --accept-flake-config "github:holochain/holonix?ref=main-0.5#hc" -- "$@"

