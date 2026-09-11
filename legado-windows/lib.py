#!/usr/bin/env python3
"""
Bibliotecário/escrivão do vault — wrapper multiplataforma.

Fluxo de uma sessão:
    1. espelha o vault   -> mirror/   (somente leitura pro agente)
    2. traz vault/claude -> out/      (retoma trabalho de outra máquina)
    3. abre o Claude Code em BASE
    4. ao fechar, devolve out/ -> vault/claude

Quem escreve no vault é ESTE script, nunca o agente. O Claude Code não tem
nenhum caminho configurado pro vault real.

Uso:
    python lib.py            sessão completa (espelha, abre, devolve)
    python lib.py --sync     só espelha e puxa, não abre nada
    python lib.py --push     só devolve out/ pro vault
    python lib.py --no-push  sessão sem devolver no fim
"""

import os
import shutil
import subprocess
import sys
from pathlib import Path

# --- config -----------------------------------------------------------------

VAULTS = {
    "nt": Path(r"C:\Users\gabri\Obsidian\vault"),      # Windows
    "posix": Path.home() / "Obsidian" / "vault",        # macOS / Linux
}

BASE = Path(r"C:\claude-lib") if os.name == "nt" else Path.home() / "claude-lib"

IGNORAR = shutil.ignore_patterns(
    ".obsidian", ".trash", ".git", "*.tmp",
    "claude",   # NAO REMOVER: senao o agente le as proprias saidas como se
                # fossem notas suas e passa a analisar a propria sombra.
    # anexos pesados: descomente se o vault crescer com midia
    # "*.png", "*.jpg", "*.pdf", "*.mp4",
)

# ----------------------------------------------------------------------------

vault = VAULTS[os.name]
mirror = BASE / "mirror"
out = BASE / "out"
destino = vault / "claude"


def checar():
    if not vault.is_dir():
        sys.exit(f"Vault nao encontrado: {vault}")

    # guarda-costas: o mirror nunca pode estar dentro do vault, senao o
    # rmtree abaixo comeria notas de verdade.
    m = mirror.resolve()
    v = vault.resolve()
    if m == v or v in m.parents:
        sys.exit("ABORTADO: o mirror esta dentro do vault. Isso apagaria suas notas.")


def espelhar():
    if mirror.exists():
        shutil.rmtree(mirror)
    shutil.copytree(vault, mirror, ignore=IGNORAR)
    out.mkdir(parents=True, exist_ok=True)

    n = sum(1 for _ in mirror.rglob("*.md"))
    kb = sum(f.stat().st_size for f in mirror.rglob("*") if f.is_file()) / 1024
    print(f"Espelho atualizado: {n} notas, {kb:.0f} KB")


def puxar():
    """vault/claude -> out/  (retoma o que ficou pela metade em outra maquina)"""
    if not destino.exists():
        return
    shutil.copytree(destino, out, dirs_exist_ok=True)
    n = sum(1 for _ in out.rglob("*.md"))
    print(f"Propostas em out/: {n}")


def empurrar():
    """out/ -> vault/claude  (so sobrepoe, nunca apaga)"""
    if not out.exists() or not any(out.iterdir()):
        print("Nada em out/ pra devolver.")
        return
    destino.mkdir(parents=True, exist_ok=True)
    shutil.copytree(out, destino, dirs_exist_ok=True)
    print(f"Propostas devolvidas pro vault: {destino}")


def abrir_claude():
    try:
        subprocess.run("claude", cwd=BASE, shell=True)
    except FileNotFoundError:
        sys.exit("Claude Code nao encontrado no PATH.")


if __name__ == "__main__":
    args = sys.argv[1:]
    checar()

    if "--push" in args:
        empurrar()
        sys.exit()

    espelhar()
    puxar()

    if "--sync" in args:
        sys.exit()

    abrir_claude()

    if "--no-push" not in args:
        empurrar()
