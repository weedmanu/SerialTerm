#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"

cd "${ROOT_DIR}"

# Prioritise rustup toolchain over any system-installed Rust/Cargo.
if [ -d "${HOME}/.cargo/bin" ]; then
    export PATH="${HOME}/.cargo/bin:${PATH}"
fi

require_command() {
	local cmd="$1"
	local hint="$2"

	if ! command -v "$cmd" >/dev/null 2>&1; then
		echo "[pre-commit] outil manquant: $cmd"
		echo "[pre-commit] correction: $hint"
		exit 1
	fi
}

require_command cargo "installer Rust et Cargo puis relancer scripts/install-deps.sh"
require_command cargo-audit "installer cargo-audit ou relancer scripts/install-deps.sh"
require_command cargo-deny "installer cargo-deny ou relancer scripts/install-deps.sh"

echo "[pre-commit] cargo fmt --all -- --check"
cargo fmt --all -- --check

echo "[pre-commit] cargo check --all-targets"
cargo check --all-targets

echo "[pre-commit] cargo clippy --all-targets --all-features -- -D warnings"
cargo clippy --all-targets --all-features -- -D warnings

echo "[pre-commit] RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --document-private-items"
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --document-private-items

if command -v xvfb-run >/dev/null 2>&1; then
	echo "[pre-commit] GDK_BACKEND=x11 LIBGL_ALWAYS_SOFTWARE=1 MESA_LOADER_DRIVER_OVERRIDE=llvmpipe xvfb-run -a cargo test --all-targets"
	GDK_BACKEND=x11 LIBGL_ALWAYS_SOFTWARE=1 MESA_LOADER_DRIVER_OVERRIDE=llvmpipe xvfb-run -a cargo test --all-targets
elif [ -n "${DISPLAY:-}" ] && [ -S "/run/user/$(id -u)/bus" ]; then
	# Display X + socket D-Bus disponibles (bureau actif) : les tests gtk4::test passent en natif.
	echo "[pre-commit] GDK_BACKEND=x11 cargo test --all-targets (display natif + D-Bus)"
	GDK_BACKEND=x11 DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$(id -u)/bus" cargo test --all-targets
else
	echo "[pre-commit] xvfb-run absent et pas de display, fallback vers cargo test --all-targets"
	cargo test --all-targets
fi

echo "[pre-commit] cargo audit -q"
cargo audit -q

echo "[pre-commit] cargo deny check"
cargo deny check

if command -v cargo-machete >/dev/null 2>&1; then
	echo "[pre-commit] cargo machete --with-metadata (dépendances inutilisées, hors vendor/)"
	# Les crates vendorisées (vendor/) ont des dépendances de test/bench non pertinentes.
	# On filtre leur section pour ne signaler que les problèmes dans notre propre code.
	machete_out=$(cargo machete --with-metadata 2>&1 || true)
	# Format des lignes coupables : "<crate> -- <path>:"
	# On n'échoue que si au moins une ligne de ce type pointe hors de vendor/.
	non_vendor_findings=$(echo "$machete_out" | grep " -- " | grep -v "vendor/" || true)
	if [ -n "$non_vendor_findings" ]; then
		echo "$machete_out" >&2
		echo "[pre-commit] cargo machete a détecté des dépendances inutilisées — corriger ou justifier." >&2
		exit 1
	fi
else
	echo "[pre-commit] cargo-machete absent — skipped (installer via scripts/install-deps.sh)"
fi

echo "[pre-commit] OK"