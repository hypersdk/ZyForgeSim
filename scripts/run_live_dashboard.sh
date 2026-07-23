#!/usr/bin/env bash
# Run the Rich live dashboard using the project venv.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [[ ! -d .venv ]]; then
  echo "No .venv found. Run ./scripts/setup_dev.sh first." >&2
  exit 1
fi

# shellcheck disable=SC1091
source .venv/bin/activate

if ! python -c "import forgesim._forgesim" 2>/dev/null; then
  echo "ForgeSim extension not built. Run ./scripts/setup_dev.sh" >&2
  exit 1
fi

if ! python -c "import rich" 2>/dev/null; then
  echo "Installing rich..."
  python -m pip install rich
fi

exec python python/examples/live_dashboard.py "$@"
