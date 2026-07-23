# ZyForgeSim (ForgeSim)

ForgeSim is a discrete-event simulator for Kubernetes-native GPU scheduling inspired by Zyvor Forge. It models clusters, MIG, topology, tenants, quotas, gang scheduling, and AI workloads, enabling scheduler development, RL research, and performance evaluation without requiring physical NVIDIA GPUs.

## Architecture

- **Rust core** — event engine, cluster model, schedulers, metrics, Forge bundle loader
- **Python API** — thin PyO3 bindings + Forge CRD adapters, Gymnasium env, visualization

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

### Scheduler policies (M6)

```bash
# Priority: highest priority first, no preemption
cargo run -p forgesim-cli -- run --config configs/clusters/priority_scheduler.yaml

# Preemptive: evict lower-priority running jobs for higher-priority arrivals
cargo run -p forgesim-cli -- run --config configs/clusters/preemption_preemptive.yaml

# Forge bundle with scheduler flag (fifo | priority | preemptive | forge | bestfit)
cargo run -p forgesim-cli -- run \
  --forge-bundle tests/fixtures/forge \
  --scheduler forge
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

### Topology + gang placement (M5 / M6)

```bash
# NVLink-domain-aware placement; cross-domain jobs inflate runtime
cargo run -p forgesim-cli -- run --config configs/clusters/topology_penalty.yaml

# Gang jobs require GPUs spread across gang_size_nodes distinct nodes
cargo run -p forgesim-cli -- run --config configs/clusters/gang_m6.yaml

# Gang timeout fails jobs that cannot be placed in time (jobs_failed metric)
cargo run -p forgesim-cli -- run --config configs/clusters/gang_timeout_m6.yaml
```

### Visualization (M8)

```bash
cargo run -p forgesim-cli -- run \
  --config configs/clusters/small_h100.yaml \
  --jobs-output outputs/jobs.json

pip install -e '.[viz]'
python python/examples/plot_run.py outputs/jobs.json
```

### Live CLI dashboard (Phase 1 UI)

Rich terminal dashboard — cluster summary, GPU utilization bars, queue list:

On macOS Homebrew Python, use the setup script if `venv` fails on `pyexpat`:

```bash
./scripts/setup_dev.sh
source .venv/bin/activate
./scripts/run_live_dashboard.sh --config configs/clusters/small_h100.yaml
```

Or manually:

```bash
python3.13 -m venv .venv
source .venv/bin/activate
pip install maturin rich pyyaml
maturin develop
python python/examples/live_dashboard.py --config configs/clusters/small_h100.yaml
```

**Important:** use the venv Python (`source .venv/bin/activate`). System `python3` (e.g. 3.9) is too old and will not have the Rust extension installed.

### Web dashboard (Phase 2 UI)

FastAPI backend + Next.js frontend — run simulations, replay scheduler decisions, compare configs:

```bash
pip install -e '.[server]'
uvicorn forgesim.server.app:app --reload --port 8080

cd web && npm install && npm run dev
```

Open http://localhost:3000. See [web/README.md](web/README.md) and [docs/ui_roadmap.md](docs/ui_roadmap.md).

### Python + RL (M7)

On macOS Homebrew Python, use the setup script if `venv` fails on `pyexpat`:

```bash
./scripts/setup_venv.sh
source .venv/bin/activate
maturin develop
pip install -e '.[rl]'
python python/examples/run_rl_env.py
python python/baselines/ppo_cleanrl.py --config configs/clusters/rl_small.yaml
```

### Test layout

| Layer | Location | What it covers |
|-------|----------|----------------|
| Rust unit | `crates/*/src/` (`#[test]` modules) | Models, MIG, resource manager, FIFO, trace parsing |
| Rust integration | `crates/forgesim-config/tests/integration.rs` | Full sim pipelines (YAML, Forge bundle, trace, MIG, RL, topology) |
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
python/forgesim/     Python package + adapters, envs, viz
configs/
  profiles/          Calibrated model runtimes (model + gpuType)
  hardware/          GPU capability profiles
tests/fixtures/forge/  Golden Forge export bundle
docs/                Architecture, milestones, Forge input mapping
```

## Milestones

See [docs/milestones.md](docs/milestones.md). **M1–M8 complete**, including topology runtime inflation, gang timeout, RL (M7), and visualization (M8).

Schedulers: `fifo`, `priority`, `preemptive`, `forge` (alias for preemptive), `bestfit`.

## Forge input

See [docs/forge_input.md](docs/forge_input.md) for CRD mapping rules, export workflow, and adapter levels.

## License

Apache-2.0 — see [LICENSE](LICENSE).
