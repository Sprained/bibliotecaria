#!/usr/bin/env bash
# Builda o vault_mcp em release e copia pra binaries/ com o sufixo de target
# triple que o externalBin do Tauri espera (mesmo tratamento que o binário
# do claude, buscado por fetch-claude-sidecar.sh).
#
# IMPORTANTE: rode fetch-claude-sidecar.sh ANTES deste, num clone limpo — o
# build.rs do Tauri valida TODOS os caminhos do externalBin (claude e
# vault_mcp) antes de compilar qualquer coisa do pacote, e só o vault_mcp
# tem o bootstrap de placeholder abaixo.
# Uso: ./scripts/build-sidecars.sh
set -euo pipefail

cd "$(dirname "$0")/.."

TARGET_TRIPLE=$(rustc --print host-tuple)
EXT=""
[[ "$TARGET_TRIPLE" == *windows* ]] && EXT=".exe"
SIDECAR="binaries/vault_mcp-${TARGET_TRIPLE}${EXT}"

# O build.rs do Tauri valida que os arquivos do externalBin já existem antes
# de compilar (mesmo só compilando o bin vault_mcp) — na primeira vez cria um
# placeholder vazio só pra passar essa checagem, depois sobrescreve com o
# binário de verdade.
mkdir -p binaries
[ -f "$SIDECAR" ] || : > "$SIDECAR"

cargo build --release --bin vault_mcp
cp "target/release/vault_mcp${EXT}" "$SIDECAR"
chmod +x "$SIDECAR" 2>/dev/null || true

echo "vault_mcp buildado pra ${TARGET_TRIPLE} em ${SIDECAR}"
