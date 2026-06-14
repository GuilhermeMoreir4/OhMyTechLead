#!/usr/bin/env bash
set -e

BOLD='\033[1m'
CYAN='\033[0;36m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
RESET='\033[0m'

print() { echo -e "${CYAN}==>${RESET} ${BOLD}$1${RESET}"; }
ok()    { echo -e "${GREEN} ✓${RESET} $1"; }
warn()  { echo -e "${YELLOW} !${RESET} $1"; }
fail()  { echo -e "${RED} ✗${RESET} $1"; exit 1; }

echo ""
echo -e "${BOLD}oh-my-tech-lead — instalador${RESET}"
echo "────────────────────────────"
echo ""

# --- Rust / cargo ---
if ! command -v cargo &>/dev/null; then
    warn "Rust não encontrado. Instalando via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
    source "$HOME/.cargo/env"
fi

RUST_VERSION=$(rustc --version 2>/dev/null || echo "desconhecida")
ok "Rust encontrado: $RUST_VERSION"

# --- Build e instalação ---
print "Compilando e instalando omtl..."
cargo install --path . --quiet
ok "omtl instalado em $(which omtl 2>/dev/null || echo '~/.cargo/bin/omtl')"

# --- Verificar PATH ---
if ! command -v omtl &>/dev/null; then
    warn "~/.cargo/bin não está no seu PATH."
    echo "   Adicione ao seu ~/.bashrc ou ~/.zshrc:"
    echo ""
    echo '   export PATH="$HOME/.cargo/bin:$PATH"'
    echo ""
    warn "Depois rode: omtl setup"
    exit 0
fi

echo ""

# --- Setup ---
read -r -p "$(echo -e "${BOLD}Configurar agora? (bot Discord, horário de envio) [S/n]:${RESET} ")" answer
answer=${answer:-S}

if [[ "$answer" =~ ^[Ss]$ ]]; then
    echo ""
    omtl setup
else
    echo ""
    ok "Instalação concluída!"
    echo ""
    echo "  Quando quiser configurar:"
    echo -e "    ${BOLD}omtl setup${RESET}   — configura bot Discord e horário"
    echo -e "    ${BOLD}omtl${RESET}         — abre a TUI"
    echo -e "    ${BOLD}omtl send${RESET}    — envia o relatório agora"
fi

echo ""
