#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="${ROOT_DIR}/logs"
STAMP="$(date +%Y%m%d_%H%M%S)"
LOG_FILE="${LOG_DIR}/soak_${STAMP}.log"

mkdir -p "${LOG_DIR}"

export RUST_LOG="${RUST_LOG:-info}"
export SERIAL_TERM_SOAK_GENERATOR="${SERIAL_TERM_SOAK_GENERATOR:-1}"
export SERIAL_TERM_SOAK_INTERVAL_MS="${SERIAL_TERM_SOAK_INTERVAL_MS:-16}"
export SERIAL_TERM_SOAK_LINES_PER_TICK="${SERIAL_TERM_SOAK_LINES_PER_TICK:-6}"
export SERIAL_TERM_SOAK_PROGRESS_WIDTH="${SERIAL_TERM_SOAK_PROGRESS_WIDTH:-40}"
export SERIAL_TERM_SOAK_DIAGNOSTICS_SECS="${SERIAL_TERM_SOAK_DIAGNOSTICS_SECS:-60}"
export SERIAL_TERM_SOAK_DURATION_SECS="${SERIAL_TERM_SOAK_DURATION_SECS:-0}"

echo "[soak] root=${ROOT_DIR}"
echo "[soak] log=${LOG_FILE}"
echo "[soak] interval_ms=${SERIAL_TERM_SOAK_INTERVAL_MS} lines_per_tick=${SERIAL_TERM_SOAK_LINES_PER_TICK} progress_width=${SERIAL_TERM_SOAK_PROGRESS_WIDTH} diagnostics_secs=${SERIAL_TERM_SOAK_DIAGNOSTICS_SECS} duration_secs=${SERIAL_TERM_SOAK_DURATION_SECS}"

cd "${ROOT_DIR}"
if [[ -z "${DISPLAY:-}" ]] && command -v xvfb-run >/dev/null 2>&1; then
	echo "[soak] no DISPLAY detected, using xvfb-run -a"
	xvfb-run -a cargo run --release 2>&1 | tee "${LOG_FILE}"
else
	cargo run --release 2>&1 | tee "${LOG_FILE}"
fi