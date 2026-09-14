#!/usr/bin/env bash
# Testa um binário de servidor MCP na mão, sem precisar do `claude` de verdade.
# Uso: ./test-mcp.sh <caminho-do-binario> <argumento-do-binario> <arquivo-de-mensagens.jsonl>
set -euo pipefail

BIN="$1"
ARG="$2"
INPUT="$3"

OUT=$(mktemp)
ERR=$(mktemp)

(cat "$INPUT"; sleep 2) | "$BIN" "$ARG" > "$OUT" 2> "$ERR" &
PID=$!
sleep 3
kill "$PID" 2>/dev/null || true

echo "--- stdout ---"
cat "$OUT"
echo "--- stderr ---"
cat "$ERR"

rm -f "$OUT" "$ERR"
