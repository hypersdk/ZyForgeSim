# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Fixed
- `forge-sim run --output` / `replay --output` no longer fail with "No such
  file or directory" when the target directory does not already exist.
- `forgesim-py` failed to compile after `mig_reconfigs` was added to
  `SimulationMetrics` (M4); the Python `SimResult` binding now exposes it too.

### Added
- CI workflows for Rust (`fmt`, `clippy`, unit + integration tests) and
  Python (`maturin build` + `unittest`).
- `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `rust-toolchain.toml`.

## [0.1.0] — M1–M4

- **M1 — Simulation core**: discrete-event engine, FIFO scheduler, internal
  YAML workload configs, CLI, Python bindings.
- **M2 — Forge compatibility**: `FabricAIJob`/`FabricGpuNode`/`FabricQuota`
  ingest via `ForgeBundleAdapter`, calibrated runtime profiles,
  `--forge-bundle` CLI flag.
- **M3 — Trace replay**: JSONL scheduler event replay with oracle vs.
  simulated placement diff reporting.
- **M4 — MIG simulation**: fractional GPU slice allocation with simulated
  reconfiguration delay.

See [docs/milestones.md](docs/milestones.md) for full success criteria per
milestone.
