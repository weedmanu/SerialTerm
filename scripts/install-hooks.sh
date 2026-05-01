#!/usr/bin/env sh
#
# Active les hooks Git versionnés du dépôt.
#
# Effets :
#   - configure `core.hooksPath` sur `.githooks/`;
#   - rend exécutables les hooks (`pre-commit`, `pre-push`, `commit-msg`)
#     et le script de validation standard.
#
# Idempotent : peut être ré-exécuté sans risque.
#
# Usage :
#   ./scripts/install-hooks.sh

set -eu

ROOT_DIR="$(cd -- "$(dirname -- "$0")/.." && pwd)"
cd "${ROOT_DIR}"

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "✗ Ce répertoire n'est pas un dépôt Git." >&2
    exit 1
fi

if [ ! -d .githooks ]; then
    echo "✗ Dossier .githooks/ introuvable." >&2
    exit 1
fi

echo "→ Configuration de core.hooksPath = .githooks"
git config core.hooksPath .githooks

echo "→ chmod +x .githooks/* et scripts de validation"
chmod +x \
    .githooks/pre-commit \
    .githooks/pre-push \
    .githooks/commit-msg \
    scripts/pre-commit-checks.sh

echo "✓ Hooks Git activés."
echo ""
echo "Vérification :"
git config --get core.hooksPath
ls -l .githooks/
