#!/usr/bin/env bash
# Start the ForgeSim Next.js frontend (port 3000 by default).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=common.sh
source "$ROOT/scripts/common.sh"

WEB="$ROOT/web"
PORT="${PORT:-3000}"

ensure_web_deps "$WEB"

echo "ForgeSim web UI → http://127.0.0.1:${PORT}"
echo "(proxies /api and /ws to http://127.0.0.1:8080 — start ./scripts/run_web_api.sh first)"
echo

cd "$WEB"
exec npm run dev -- -p "$PORT"
