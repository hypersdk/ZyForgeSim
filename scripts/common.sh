#!/usr/bin/env bash
# Shared helpers for ForgeSim shell scripts.
fix_homebrew_pyexpat() {
  if [[ -d /opt/homebrew/opt/expat/lib ]]; then
    export DYLD_LIBRARY_PATH="/opt/homebrew/opt/expat/lib${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
  elif [[ -d /usr/local/opt/expat/lib ]]; then
    export DYLD_LIBRARY_PATH="/usr/local/opt/expat/lib${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
  fi
}

forgesim_root() {
  cd "$(dirname "${BASH_SOURCE[1]}")/.." && pwd
}

require_venv() {
  local root="$1"
  local py="$root/.venv/bin/python"
  if [[ ! -x "$py" ]]; then
    echo "No .venv found. Run ./scripts/setup_dev.sh first." >&2
    exit 1
  fi
  if ! "$py" -c "import pip" >/dev/null 2>&1; then
    echo ".venv is incomplete (no pip). Run ./scripts/setup_dev.sh" >&2
    exit 1
  fi
}

ensure_server_deps() {
  local py="$1"
  if ! "$py" -c "import fastapi, uvicorn" >/dev/null 2>&1; then
    echo "Installing server deps (fastapi, uvicorn, websockets, pydantic)..."
    if command -v uv >/dev/null 2>&1; then
      uv pip install --python "$py" fastapi "uvicorn[standard]" websockets pydantic rich pyyaml
    else
      "$py" -m pip install fastapi "uvicorn[standard]" websockets pydantic rich pyyaml
    fi
  fi
}

ensure_forge_extension() {
  local py="$1"
  export PYTHONPATH="${2}/python${PYTHONPATH:+:$PYTHONPATH}"
  if ! "$py" -c "import forgesim._forgesim" >/dev/null 2>&1; then
    echo "ForgeSim Rust extension not built. Run ./scripts/setup_dev.sh" >&2
    exit 1
  fi
}

ensure_web_deps() {
  local web_dir="$1"
  if [[ ! -d "$web_dir/node_modules" ]]; then
    echo "Installing web deps (npm install)..."
    (cd "$web_dir" && npm install)
  fi
}
