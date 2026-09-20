#!/usr/bin/env bash
# Busca o binário oficial do claude (via npm pack, sem instalar o CLI
# inteiro) pro target triple pedido e coloca em binaries/ com o nome que o
# externalBin do Tauri espera.
# Uso: ./scripts/fetch-claude-sidecar.sh <target-triple>
set -euo pipefail

cd "$(dirname "$0")/.."

TARGET_TRIPLE="${1:?uso: fetch-claude-sidecar.sh <target-triple>}"

case "$TARGET_TRIPLE" in
  aarch64-apple-darwin) NPM_PKG="darwin-arm64"; BIN_NAME="claude" ;;
  x86_64-apple-darwin) NPM_PKG="darwin-x64"; BIN_NAME="claude" ;;
  x86_64-unknown-linux-gnu) NPM_PKG="linux-x64"; BIN_NAME="claude" ;;
  aarch64-unknown-linux-gnu) NPM_PKG="linux-arm64"; BIN_NAME="claude" ;;
  x86_64-pc-windows-msvc) NPM_PKG="win32-x64"; BIN_NAME="claude.exe" ;;
  aarch64-pc-windows-msvc) NPM_PKG="win32-arm64"; BIN_NAME="claude.exe" ;;
  *)
    echo "target triple não mapeado: $TARGET_TRIPLE" >&2
    exit 1
    ;;
esac

SIDECAR="binaries/claude-${TARGET_TRIPLE}"
[[ "$BIN_NAME" == *.exe ]] && SIDECAR="${SIDECAR}.exe"

WORKDIR=$(mktemp -d)
trap 'rm -rf "$WORKDIR"' EXIT

npm pack "@anthropic-ai/claude-code-${NPM_PKG}" --pack-destination "$WORKDIR" --silent
TARBALL=$(ls "$WORKDIR"/*.tgz)
tar -xzf "$TARBALL" -C "$WORKDIR"

mkdir -p binaries
cp "$WORKDIR/package/$BIN_NAME" "$SIDECAR"
chmod +x "$SIDECAR"

echo "claude baixado pra ${TARGET_TRIPLE} em ${SIDECAR}"
