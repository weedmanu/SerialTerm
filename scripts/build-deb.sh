#!/bin/bash
#
# Script de création du paquet Debian pour SerialTerm
# 
# Prérequis :
#  - build-essential
#  - debhelper
#  - cargo (Rust)
#  - libgtk-4-dev libadwaita-1-dev pkgconf
#
# Usage :
#   ./scripts/build-deb.sh
#
# Le .deb sera généré dans le répertoire dist/debian

set -e

cd "$(dirname "$0")/.."

PKG_NAME="serial-term"
OUT_DIR="dist/debian"
APP_VERSION="$(awk '
    /^\[package\]/ { in_package = 1; next }
    /^\[/ && in_package { exit }
    in_package && /^version[[:space:]]*=/ {
        gsub(/"/, "", $3);
        print $3;
        exit;
    }
' Cargo.toml)"
if [ -z "$APP_VERSION" ]; then
    echo "✗ Erreur : impossible de déterminer la version depuis Cargo.toml" >&2
    exit 1
fi
mkdir -p "$OUT_DIR"

echo "═══════════════════════════════════════════════════════════"
echo "  SerialTerm - Construction du paquet Debian (.deb)"
echo "═══════════════════════════════════════════════════════════"

# Vérifier les prérequis
echo "✓ Vérification des prérequis..."

for cmd in cargo debuild lintian; do
    if ! command -v "$cmd" &> /dev/null; then
        echo "✗ Erreur : '$cmd' n'est pas installé"
        echo ""
        echo "Installation sur Ubuntu/Debian :"
        echo "  sudo apt install build-essential debhelper devscripts cargo lintian pkgconf"
        exit 1
    fi
done

# Vérifier les dépendances de développement
echo "✓ Dépendances de développement : OK"

# Nettoyer les builds antérieurs
echo ""
echo "📦 Nettoyage des builds antérieurs..."

rm -f ../${PKG_NAME}_* 2>/dev/null || true
rm -f "$OUT_DIR"/*.deb 2>/dev/null || true

# Compiler le projet en release
echo ""
echo "🔨 Compilation en mode release (cela peut prendre quelques secondes)..."
cargo build --release 2>&1 | grep -E "Compiling serial-term|Finished" || true

# Créer le paquet avec debuild
echo ""
echo "📋 Création du paquet Debian avec debuild..."
echo ""

debuild -us -uc --lintian-opts --suppress-tags=bad-distribution-in-changes-file

echo ""
echo "🧹 Déplacement des paquets et purge des caches..."
mv ../${PKG_NAME}_*.deb "$OUT_DIR/"

dh_clean 2>/dev/null || true
rm -f ../${PKG_NAME}_* 2>/dev/null || true

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "✓ Succès ! Le paquet .deb a été créé."
echo ""
echo "📁 Fichier généré:"
ls -lh "$OUT_DIR"/*.deb | tail -1 | awk '{print "   " $9 " (" $5 ")"}'
echo ""
echo "Installation :"
echo "  sudo dpkg -i dist/debian/serial-term_${APP_VERSION}*.deb"
echo ""
echo "Désinstallation :"
echo "  sudo apt remove serial-term"
echo "═══════════════════════════════════════════════════════════"
