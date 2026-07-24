#!/usr/bin/env bash
# Shared helpers for ForgeSim shell scripts.
fix_homebrew_pyexpat() {
  if [[ -d /opt/homebrew/opt/expat/lib ]]; then
    export DYLD_LIBRARY_PATH="/opt/homebrew/opt/expat/lib${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
  elif [[ -d /usr/local/opt/expat/lib ]]; then
    export DYLD_LIBRARY_PATH="/usr/local/opt/expat/lib${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
  fi
}

# Homebrew python@3.13 links pyexpat against Homebrew expat but the loader often
# picks /usr/lib/libexpat.1.dylib. Wrap venv interpreters so pip/maturin inherit
# the correct library path even when DYLD_* is stripped from the parent shell.
wrap_venv_python_for_pyexpat() {
  local venv="${1:-}"
  [[ -n "$venv" ]] || return 0
  local py313="$venv/bin/python3.13"
  [[ -e "$py313" ]] || return 0

  local real_py=""
  if [[ -L "$py313" ]]; then
    real_py="$(readlink "$py313")"
  elif [[ -f "$py313.real" ]]; then
    real_py="$(readlink "$py313.real" 2>/dev/null || true)"
    [[ -n "$real_py" ]] || real_py="$py313.real"
  else
    return 0
  fi

  if [[ "$real_py" != /* ]]; then
    real_py="$venv/bin/$real_py"
  fi

  if [[ -f "$py313" && ! -L "$py313" ]] && head -1 "$py313" | grep -q "ForgeSim: Homebrew pyexpat wrapper"; then
    return 0
  fi

  mv -f "$py313" "${py313}.real"
  cat >"$py313" <<'EOF'
#!/usr/bin/env bash
# ForgeSim: Homebrew pyexpat wrapper
if [[ -d /opt/homebrew/opt/expat/lib ]]; then
  export DYLD_LIBRARY_PATH="/opt/homebrew/opt/expat/lib${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
elif [[ -d /usr/local/opt/expat/lib ]]; then
  export DYLD_LIBRARY_PATH="/usr/local/opt/expat/lib${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
fi
exec "$(dirname "$0")/python3.13.real" "$@"
EOF
  chmod +x "$py313"
}

ensure_pip_shims() {
  local venv="${1:-}"
  [[ -n "$venv" ]] || return 0
  cat >"$venv/bin/pip" <<'EOF'
#!/usr/bin/env bash
if [[ -d /opt/homebrew/opt/expat/lib ]]; then
  export DYLD_LIBRARY_PATH="/opt/homebrew/opt/expat/lib${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
elif [[ -d /usr/local/opt/expat/lib ]]; then
  export DYLD_LIBRARY_PATH="/usr/local/opt/expat/lib${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
fi
exec "$(dirname "$0")/python" -m pip "$@"
EOF
  chmod +x "$venv/bin/pip"
  ln -sf pip "$venv/bin/pip3"
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
