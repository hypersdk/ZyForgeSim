#!/usr/bin/env bash
# Start FastAPI backend + Next.js frontend for the ForgeSim web dashboard.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=common.sh
source "$ROOT/scripts/common.sh"

API_PORT="${API_PORT:-8080}"
UI_PORT="${UI_PORT:-3000}"

fix_homebrew_pyexpat
require_venv "$ROOT"

PY="$ROOT/.venv/bin/python"
ensure_forge_extension "$PY" "$ROOT"
ensure_server_deps "$PY"
ensure_web_deps "$ROOT/web"

export PYTHONPATH="$ROOT/python${PYTHONPATH:+:$PYTHONPATH}"

cleanup() {
  if [[ -n "${API_PID:-}" ]] && kill -0 "$API_PID" 2>/dev/null; then
    kill "$API_PID" 2>/dev/null || true
    wait "$API_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

echo "Starting ForgeSim web dashboard..."
echo "  API: http://127.0.0.1:${API_PORT}"
echo "  UI:  http://127.0.0.1:${UI_PORT}"
echo
echo "Press Ctrl+C to stop both servers."
echo

"$PY" -m uvicorn forgesim.server.app:app --reload --host 127.0.0.1 --port "$API_PORT" &
API_PID=$!

# Give the API a moment to bind.
sleep 1

cd "$ROOT/web"
exec npm run dev -- -p "$UI_PORT"
