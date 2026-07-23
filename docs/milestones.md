# ForgeSim Milestones

| Milestone | Status | Deliverable |
|-----------|--------|-------------|
| **M1 — Simulation core** | Done | DES, FIFO, internal YAML, CLI, Python bindings |
| **M2 — Forge Compatibility** | Done | Multi-CRD ingest, corrected mappings, profiles, `--forge-bundle` CLI |
| **M3 — Trace replay** | Done | Scheduler event JSONL replay + oracle vs FIFO diff report |
| **M4 — MIG simulation** | Done | MIG slice partition/reconfig with simulated delay |
| **M5 — Topology** | [Scoped](design/m5_topology.md) | NVLink/PCIe graph from FabricGpuNode topology |
| **M6 — Forge scheduler features** | [Scoped](design/m6_scheduler_features.md) | Quotas enforced, priority, gang plugin parity, preemption |
| **M7 — RL** | Planned | Gymnasium wrapper, PPO baselines |
| **M8 — Visualization** | Planned | Gantt, heatmaps, notebooks |

## M2 success criteria

- [x] `ForgeBundleAdapter` loads `FabricAIJob`, `FabricGpuNode`, `FabricQuota`
- [x] Correct GPU count for distributed/gang jobs (32 not 8)
- [x] Tenant resolved from `FabricQuota`, not job spec
- [x] Calibrated profiles with fail-on-missing runtime
- [x] `forge-sim run --forge-bundle` CLI
- [x] Golden fixtures in `tests/fixtures/forge/`
- [x] Export workflow documented in `docs/forge_input.md`

## M3 success criteria

- [x] JSONL trace format with `JobSubmitted` / `JobScheduled` events
- [x] `TraceAdapter` (Python) + `forgesim-config::trace` (Rust)
- [x] `forge-sim replay --trace` CLI with cluster from config or forge bundle
- [x] Oracle vs simulated placement diff report JSON
- [x] Golden fixture `tests/fixtures/traces/fifo_match.jsonl`

## Quick commands

```bash
# Internal synthetic workload (M1)
cargo run -p forgesim-cli -- run --config configs/clusters/small_h100.yaml

# Forge export bundle (M2)
cargo run -p forgesim-cli -- run --forge-bundle tests/fixtures/forge --profiles-dir configs/profiles

# Trace replay + decision diff (M3)
cargo run -p forgesim-cli -- replay \
  --trace tests/fixtures/traces/fifo_match.jsonl \
  --config configs/clusters/single_gpu.yaml
```

## Running tests

```bash
# Rust unit tests (all crates)
cargo test --workspace --exclude forgesim-py

# Rust integration tests
cargo test -p forgesim-config --test integration
cargo test -p forgesim-cli --test cli_integration

# Python unit + integration tests
PYTHONPATH=python python3 -m unittest discover -s python/tests -v
```

### MIG simulation (M4)

```bash
cargo run -p forgesim-cli -- run --config configs/clusters/mig_single.yaml
```

## M4 success criteria

- [x] MIG profiles in `configs/mig/` (H100 1g/2g/3g/7g)
- [x] Jobs with `mig_profile` + `mig_count` allocate slices, not whole GPUs
- [x] Reconfiguration delay simulated (`reconfig_seconds: 30`)
- [x] `mig_reconfigs` tracked in metrics output
- [x] Forge `spec.mig.profile/count` mapped at ingest (M2) and simulated (M4)
