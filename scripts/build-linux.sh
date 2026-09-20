#!/usr/bin/env bash
# Builda o app pra Linux (x86_64) num container Docker/OrbStack, sem
# precisar instalar nada de Linux no host. Gera .deb/.AppImage em
# app/src-tauri/target/release/bundle/.
# Uso: ./scripts/build-linux.sh
set -euo pipefail

cd "$(dirname "$0")/.."

docker build --platform linux/amd64 -t bibliotecaria-linux-build -f Dockerfile.linux .

docker run --rm --platform linux/amd64 -v "$(pwd):/workspace" bibliotecaria-linux-build bash -c "
  set -euo pipefail
  npm install
  ./src-tauri/scripts/fetch-claude-sidecar.sh x86_64-unknown-linux-gnu
  ./src-tauri/scripts/build-sidecars.sh
  npm run tauri build -- --bundles deb,appimage
"
