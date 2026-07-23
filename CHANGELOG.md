# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Fixed
- `forge-sim run --output` / `replay --output` no longer fail with "No such
  file or directory" when the target directory does not already exist.
- `forgesim-py` failed to compile after `mig_reconfigs` was added to
  `SimulationMetrics` (M4); the Python `SimResult` binding now exposes it too.
- A stale `JobComplete` event scheduled before a job was preempted could,
  if that job was later resumed, fire after the resumed run had already
  started — incorrectly finishing it early and freeing its GPU while the
  resumed run was still actually using it. Fixed with a
  `Job::run_generation` counter that lets the engine tell a stale
  completion apart from the one that matches the job's current run.
- `--forge-bundle` ingest silently found zero jobs/nodes/quotas when fed
  the output of `kubectl get <resource> -A -o yaml` — exactly the export
  command `docs/forge_input.md` documents. `kubectl` wraps multiple
  resources in a single `kind: List` document with an `items:` array
  instead of `---`-separating them; the YAML parser only ever split on
  `---` and never unwrapped `List`. Found by exporting and replaying a
  real Forge deployment. `yaml_documents()` now unwraps `kind: List`.

### Added
- CI workflows for Rust (`fmt`, `clippy`, unit + integration tests) and
  Python (`maturin build` + `unittest`).
- `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `rust-toolchain.toml`.
- M6 (quotas slice): `FabricQuota.spec.gpuQuota.maxGPUs` is now enforced
  per tenant at placement time — a job that would push its tenant over
  quota stays queued until another of that tenant's jobs frees capacity.
  Internal YAML configs can set the same limits via
  `cluster.tenant_quotas`. See `docs/forge_input.md` and
  `docs/design/m6_scheduler_features.md`.
- M6 (priority scheduler slice): `PriorityScheduler` now really schedules
  (previously a no-op stub) — orders the waiting queue by highest
  `priority` first, ties broken by earliest arrival. Select it with
  `scheduler.type: priority` in internal YAML configs or `--scheduler
  priority` on `forge-sim run --forge-bundle` / `forge-sim replay`. Does
  not preempt already-running jobs.
- M6 (preemption slice): new `PreemptivePriorityScheduler`
  (`scheduler.type: preemptive` / `--scheduler preemptive`) — a waiting
  job may evict lower-priority running jobs to fit. Evicted jobs resume
  later with their remaining runtime (no restart penalty), and become
  exempt from further preemption after 3 evictions. New
  `SimulationMetrics.preemptions` field, printed by the CLI as
  `preemptions:` when nonzero. See `docs/forge_input.md` and
  `docs/design/m6_scheduler_features.md`.

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
