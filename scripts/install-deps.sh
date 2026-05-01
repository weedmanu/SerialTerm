#!/bin/bash
#
# Script d'installation des dépendances pour SerialTerm
#
# Ce script installe les dépendances de build et runtime nécessaires
# pour compiler et utiliser SerialTerm sur Ubuntu/Debian.
#

# Si le script est lancé via `sh`, rebascule vers bash (bashisms utilisés plus bas).
if [ -z "${BASH_VERSION:-}" ]; then
    exec bash "$0" "$@"
fi

set -e

# Prioriser rustup si présent pour éviter les décalages avec un cargo système.
if [ -d "${HOME}/.cargo/bin" ]; then
    export PATH="${HOME}/.cargo/bin:${PATH}"
fi

install_cargo_tool() {
    local binary="$1"
    local package="$2"

    if command -v "$binary" >/dev/null 2>&1; then
        echo "   ✓ $binary est déjà installé"
    else
        echo "   ↻ Installation de $package (--locked)"
        if ! cargo install --locked "$package"; then
            echo "   ⚠ Échec avec --locked, nouvelle tentative sans --locked"
            cargo install "$package"
        fi
    fi
}

echo "═══════════════════════════════════════════════════════════"
echo "  SerialTerm - Installation des dépendances"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Détecter la distribution
if [ -f /etc/os-release ]; then
    . /etc/os-release
    DISTRO=$ID
else
    echo "✗ Impossible de détecter la distribution"
    exit 1
fi

echo "📦 Distribution détectée : $DISTRO"
echo ""

# Installer les dépendances selon la distribution
case "$DISTRO" in
    ubuntu|debian)
        echo "📥 Installation des dépendances pour Debian/Ubuntu..."
        echo ""
        
        # Mettre à jour les listes
        echo "↻ Mise à jour des listes de paquets..."
        sudo apt update
        
        # Build essentials + outils Debian packaging
        echo ""
        echo "📦 Build essentials & packaging..."
        sudo apt install -y build-essential debhelper devscripts lintian cargo

        # Dépendances système pour la compilation des outils Cargo (ex: cargo-outdated)
        echo ""
        echo "📦 Toolchain système (OpenSSL/CMake)..."
        sudo apt install -y libssl-dev cmake
        
        # Rust (si pas présent)
        echo ""
        echo "📦 Vérification de Rust..."
        if ! command -v cargo &> /dev/null; then
            echo "   Rust n'est pas installé. Installation de rustup..."
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            source "$HOME/.cargo/env"
        else
            echo "   ✓ Rust est déjà installé"
        fi

        if command -v rustup &> /dev/null; then
            echo "   ↻ Installation des composants Rust requis..."
            rustup component add rustfmt clippy llvm-tools rust-src
        fi
        
        # Dépendances GTK4/Libadwaita + libs de dev explicites
        # (évite les régressions si des dépendances transitives changent)
        echo ""
        echo "📦 Dépendances GTK4..."
        sudo apt install -y \
            libgtk-4-dev \
            libadwaita-1-dev \
            libglib2.0-dev \
            libgio-2.0-dev \
            libcairo2-dev \
            libpango1.0-dev \
            libgdk-pixbuf-2.0-dev \
            libgraphene-1.0-dev

        # Outils de build
        echo ""
        echo "📦 Outils..."
        sudo apt install -y pkg-config libudev-dev xvfb

        echo ""
        echo "📦 Outils Cargo de validation..."
        install_cargo_tool cargo-audit cargo-audit
        install_cargo_tool cargo-deny cargo-deny
        install_cargo_tool cargo-nextest cargo-nextest
        install_cargo_tool cargo-llvm-cov cargo-llvm-cov
        install_cargo_tool cargo-machete cargo-machete
        install_cargo_tool cargo-outdated cargo-outdated
        install_cargo_tool cargo-deb cargo-deb

        # Vérification rapide des .pc critiques utilisés pendant cargo check/clippy/test
        echo ""
        echo "📦 Vérification pkg-config (GTK/GLib)..."
        pkg-config --exists gtk4 glib-2.0 gio-2.0 gobject-2.0 pango cairo gdk-pixbuf-2.0 graphene-gobject-1.0 || {
            echo "✗ Des dépendances pkg-config GTK/GLib sont manquantes." >&2
            echo "  Relancer ce script et vérifier les paquets dev installés ci-dessus." >&2
            exit 1
        }
        
        ;;
    fedora)
        echo "📥 Installation des dépendances pour Fedora..."
        echo ""
        
        echo "↻ Mise à jour..."
        sudo dnf upgrade -y
        
        echo "📦 Build essentials..."
        sudo dnf groupinstall -y "Development Tools"
        sudo dnf install -y rpm-build
        
        if ! command -v cargo &> /dev/null; then
            echo "   Installation de rustup..."
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            source "$HOME/.cargo/env"
        fi
        
        echo "📦 Dépendances GTK4..."
        sudo dnf install -y gtk4-devel libadwaita-devel
        
        echo "📦 Outils..."
        sudo dnf install -y pkg-config xorg-x11-server-Xvfb
        
        ;;
    arch)
        echo "📥 Installation des dépendances pour Arch..."
        echo ""
        
        echo "📦 Build essentials..."
        sudo pacman -Sy --noconfirm base-devel
        
        if ! command -v cargo &> /dev/null; then
            echo "Installation de rustup..."
            sudo pacman -Sy --noconfirm rustup
            rustup default stable
        fi
        
        echo "📦 Dépendances GTK4..."
        sudo pacman -Sy --noconfirm gtk4 libadwaita
        
        echo "📦 Outils..."
        sudo pacman -Sy --noconfirm pkg-config xorg-server-xvfb
        
        ;;
    *)
        echo "⚠ Distribution non reconnue : $DISTRO"
        echo ""
        echo "Installation manuelle requise. Installez :"
        echo "  - build-essential ou equivalent"
        echo "  - Rust (via rustup)"
        echo "  - libgtk-4-dev libadwaita-1-dev (ou équivalent)"
        echo "  - pkg-config"
        echo "  - xvfb/Xvfb pour les tests GTK headless"
        echo "  - cargo-audit et cargo-deny pour les contrôles supply-chain"
        echo "  - cargo-nextest, cargo-llvm-cov, cargo-machete, cargo-outdated, cargo-deb selon le flux retenu"
        exit 1
        ;;
esac

# Activation automatique des hooks Git versionnés (idempotent).
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
if [ -x "${SCRIPT_DIR}/install-hooks.sh" ] && [ -d "${SCRIPT_DIR}/../.git" ]; then
    echo ""
    echo "📦 Activation des hooks Git versionnés..."
    "${SCRIPT_DIR}/install-hooks.sh" || echo "⚠ Activation des hooks ignorée (à exécuter manuellement : ./scripts/install-hooks.sh)"
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "✓ Toutes les dépendances sont installées !"
echo ""
echo "Étapes suivantes :"
echo "  1. cd $(dirname "$0")/.."
echo "  2. cargo build --release"
echo "  3. ./target/release/serial-term"
echo "  4. (si ignoré ci-dessus) ./scripts/install-hooks.sh"
echo ""
echo "Pour créer un paquet .deb :"
echo "  ./scripts/build-deb.sh"
echo "Pour lancer la batterie pré-commit :"
echo "  bash scripts/pre-commit-checks.sh"
echo "═══════════════════════════════════════════════════════════"
