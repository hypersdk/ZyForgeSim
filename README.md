# ZyForgeSim (ForgeSim)

ForgeSim is a discrete-event simulator for Kubernetes-native GPU scheduling inspired by Zyvor Forge. It models clusters, MIG, topology, tenants, quotas, gang scheduling, and AI workloads, enabling scheduler development, RL research, and performance evaluation without requiring physical NVIDIA GPUs.

## Architecture

- **Rust core** — event engine, cluster model, schedulers, metrics, Forge bundle loader
- **Python API** — thin PyO3 bindings + Forge CRD adapters

## Quick start

### Internal workload (M1)

```bash
cargo run -p forgesim-cli -- run --config configs/clusters/small_h100.yaml
```

### Forge export bundle (M2 — test Forge without GPUs)

1. Export from a Forge cluster:

```bash
mkdir -p forge-export/{jobs,cluster,quotas}
kubectl get fabricaijobs -A -o yaml > forge-export/jobs/all.yaml
kubectl get fabricgpunodes -o yaml > forge-export/cluster/nodes.yaml
kubectl get fabricquotas -A -o yaml > forge-export/quotas/all.yaml
```

2. Add calibrated runtime profiles in `configs/profiles/` (see `configs/profiles/gpt-13b.yaml`).

3. Run simulation:

```bash
cargo run -p forgesim-cli -- run \
  --forge-bundle forge-export \
  --profiles-dir configs/profiles
```

Or use the included fixture:

```bash
cargo run -p forgesim-cli -- run \
  --forge-bundle tests/fixtures/forge \
  --profiles-dir configs/profiles
```

### Scheduler trace replay (M3 — compare vs production Forge)

```bash
cargo run -p forgesim-cli -- replay \
  --trace tests/fixtures/traces/fifo_match.jsonl \
  --config configs/clusters/single_gpu.yaml
```

Writes `outputs/trace_diff.json` with oracle vs FIFO placement diffs.

### MIG simulation (M4)

```bash
cargo run -p forgesim-cli -- run --config configs/clusters/mig_single.yaml
```

MIG jobs use `mig_profile` and `mig_count` (Forge `spec.mig`) to allocate fractional GPU slices with a simulated reconfiguration delay.

### Python

```bash
python3 -m venv .venv && source .venv/bin/activate
maturin build --release
pip install target/wheels/forgesim-*.whl pyyaml
python3 -m unittest discover -s python/tests -v
```

### Test layout

| Layer | Location | What it covers |
|-------|----------|----------------|
| Rust unit | `crates/*/src/` (`#[test]` modules) | Models, MIG, resource manager, FIFO, trace parsing |
| Rust integration | `crates/forgesim-config/tests/integration.rs` | Full sim pipelines (YAML, Forge bundle, trace, MIG) |
| CLI integration | `crates/forgesim-cli/tests/cli_integration.rs` | `forge-sim run` / `replay` binary |
| Python unit | `python/tests/test_unit_adapters.py` | CRD mapping, profiles, bundle, trace adapters |
| Python integration | `python/tests/test_integration_cli.py` | CLI via `cargo run -p forgesim-cli` |

```bash
cargo test --workspace --exclude forgesim-py
cargo test -p forgesim-config --test integration
cargo test -p forgesim-cli --test cli_integration
PYTHONPATH=python python3 -m unittest discover -s python/tests -v
```

## Project layout

```
crates/              Rust workspace (core, scheduler, config, metrics, cli, py)
python/forgesim/     Python package + adapters
configs/
  profiles/          Calibrated model runtimes (model + gpuType)
  hardware/          GPU capability profiles
tests/fixtures/forge/  Golden Forge export bundle
docs/                Architecture, milestones, Forge input mapping
```

## Milestones

See [docs/milestones.md](docs/milestones.md). M1–M4 are complete (simulation core, Forge compatibility, trace replay, MIG simulation).

## Forge input

See [docs/forge_input.md](docs/forge_input.md) for CRD mapping rules, export workflow, and adapter levels.

## License

Apache-2.0 — see [LICENSE](LICENSE).
