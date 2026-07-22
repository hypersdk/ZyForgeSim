# Contributing to ForgeSim

## Setup

```bash
# Rust workspace
cargo build --workspace --exclude forgesim-py

# Python bindings
python3 -m venv .venv && source .venv/bin/activate
pip install "maturin>=1.7,<2.0" pyyaml
maturin build --release
pip install target/wheels/forgesim-*.whl
```

## Before opening a PR

```bash
cargo fmt --all
cargo clippy --workspace --exclude forgesim-py -- -D warnings
cargo test --workspace --exclude forgesim-py
cargo test -p forgesim-config --test integration
cargo test -p forgesim-cli --test cli_integration
python3 -m unittest discover -s python/tests -v
```

CI (`.github/workflows/rust.yml`, `python.yml`) runs the same checks on every push and PR to `main`.

## Project layout

See [docs/architecture.md](docs/architecture.md) for the crate layout and simulation loop, and [docs/forge_input.md](docs/forge_input.md) for how Forge CRDs map to internal models.

## Style

- Rust: standard `rustfmt` formatting, no `clippy` warnings.
- Prefer small, focused PRs tied to one milestone or fix — see [docs/milestones.md](docs/milestones.md) for current scope.
- New Forge field mappings must be documented in `docs/forge_input.md`'s field mapping table.

## Reporting issues

Open a GitHub issue with a minimal repro (config/bundle/trace fixture if possible).
