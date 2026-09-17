#!/usr/bin/env bash
# Builda o vault_mcp em release e copia pra binaries/ com o sufixo de target
# triple que o externalBin do Tauri espera (mesmo tratamento que já existe
# pro binário do claude, baixado manualmente).
# Uso: ./scripts/build-sidecars.sh
set -euo pipefail

cd "$(dirname "$0")/.."

TARGET_TRIPLE=$(rustc --print host-tuple)
SIDECAR="binaries/vault_mcp-${TARGET_TRIPLE}"

# O build.rs do Tauri valida que os arquivos do externalBin já existem antes
# de compilar (mesmo só compilando o bin vault_mcp) — na primeira vez cria um
# placeholder vazio só pra passar essa checagem, depois sobrescreve com o
# binário de verdade.
mkdir -p binaries
[ -f "$SIDECAR" ] || : > "$SIDECAR"

cargo build --release --bin vault_mcp
cp "target/release/vault_mcp" "$SIDECAR"
chmod +x "$SIDECAR"

echo "vault_mcp buildado pra ${TARGET_TRIPLE} em ${SIDECAR}"
