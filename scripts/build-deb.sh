#!/bin/bash
#
# Script de création du paquet Debian pour SerialTerm
# 
# Prérequis :
#  - build-essential
#  - debhelper
#  - cargo (Rust)
#  - libgtk-4-dev libadwaita-1-dev libudev-dev pkgconf
#
# Usage :
#   ./scripts/build-deb.sh
#
# Le .deb sera généré dans le répertoire dist/debian

set -euo pipefail

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
    echo "Erreur : impossible de déterminer la version depuis Cargo.toml" >&2
    exit 1
fi
DEB_VERSION="$(dpkg-parsechangelog -S Version)"
if [ "$APP_VERSION" != "$DEB_VERSION" ]; then
    echo "Versions incohérentes : Cargo $APP_VERSION, Debian $DEB_VERSION." >&2
    exit 1
fi
mkdir -p "$OUT_DIR"

echo "-----------------------------------------------------------"
echo "  SerialTerm - Construction du paquet Debian (.deb)"
echo "-----------------------------------------------------------"

# Vérifier les prérequis
echo "OK : Vérification des prérequis..."

for cmd in cargo debuild lintian; do
    if ! command -v "$cmd" &> /dev/null; then
        echo "Erreur : '$cmd' n'est pas installé"
        echo ""
        echo "Installation sur Ubuntu/Debian :"
        echo "  sudo apt install build-essential debhelper devscripts cargo lintian pkgconf"
        exit 1
    fi
done

# debian/rules compile le binaire depuis Cargo.lock.
debuild -b -us -uc

ARCH="$(dpkg-architecture -qDEB_HOST_ARCH)"
PACKAGE="${PKG_NAME}_${APP_VERSION}_${ARCH}.deb"
mv "../$PACKAGE" "$OUT_DIR/$PACKAGE"
dh_clean

echo ""
echo "-----------------------------------------------------------"
echo "Succès : Le paquet .deb a été créé."
echo ""
echo " Fichier généré:"
ls -lh "$OUT_DIR/$PACKAGE"
echo ""
echo "Installation :"
echo "  sudo apt install ./$OUT_DIR/$PACKAGE"
echo ""
echo "Désinstallation :"
echo "  sudo apt remove serial-term"
echo "-----------------------------------------------------------"
