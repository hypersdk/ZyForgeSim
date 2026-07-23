#!/usr/bin/env bash
# Create/fix the dev venv, build the Rust extension, and install dashboard deps.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PYTHON="${PYTHON:-python3.13}"
if ! command -v "$PYTHON" >/dev/null 2>&1; then
  PYTHON=python3
fi

echo "Using interpreter: $($PYTHON --version 2>&1)"

if [[ ! -d .venv ]]; then
  echo "Creating .venv..."
  "$PYTHON" -m venv .venv
fi

# shellcheck disable=SC1091
source .venv/bin/activate

python -m pip install --upgrade pip setuptools wheel maturin
python -m pip install rich pyyaml

# Build/install PyO3 extension into the venv (editable).
maturin develop

echo
echo "Dev environment ready. Activate with:"
echo "  source .venv/bin/activate"
echo
echo "Run the live dashboard:"
echo "  ./scripts/run_live_dashboard.sh --config configs/clusters/small_h100.yaml"
