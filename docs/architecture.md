# ForgeSim Architecture

## Overview

ForgeSim is a discrete-event GPU cluster scheduler simulator inspired by Zyvor Forge. It separates a high-performance **Rust simulation core** from a thin **Python research API**.

## Layers

```
Python (Gymnasium, notebooks, viz)
        │
   PyO3 / maturin
        │
Rust workspace
  ├── forgesim-core      Event engine, cluster, resources, RL session
  ├── forgesim-scheduler Scheduling policies
  ├── forgesim-config    YAML / Forge bundle / trace loaders
  ├── forgesim-metrics   Makespan, wait, utilization, timeline export
  ├── forgesim-cli       forge-sim binary
  └── forgesim-py        Python bindings (SimResult, SimSession)
```

## Simulation loop

1. Jobs arrive via `JobArrival` events
2. Scheduler selects waiting jobs and allocates GPUs (all-or-nothing); a
   preemptive scheduler may also evict lower-priority running jobs back
   into the waiting queue to make room
3. `ResourceManager` enforces tenant quotas, gang node spread, and NVLink-domain
   locality (with scatter fallback tracked as `topology_penalties`)
4. `JobComplete` events free resources and trigger re-scheduling — each
   carries the `Job::run_generation` it completes, so a stale event from a
   run that was preempted before finishing is ignored rather than
   corrupting the job's later, actual completion
5. Clock advances only to the next event (no polling)

## RL session (M7)

`RlSession` pauses the DES at scheduling decision points. An agent picks a
waiting job index (or noop); the session places it, advances time to the
next event, and returns a feature-vector observation plus wait-reduction
reward. Exposed to Python as `SimSession` and wrapped by `ForgeSimEnv`.

## Visualization (M8)

`SimulationReport` bundles aggregate metrics with a `JobsTimeline` JSON
(finished, running, and waiting jobs). The CLI writes it via
`--jobs-output`; Python `forgesim.viz` renders Gantt charts and GPU
utilization heatmaps.

## Design invariants

- The Rust core never depends on Python or Gymnasium
- Schedulers share a common `Scheduler` trait for benchmarking
- Forge CRDs and traces convert to internal models via adapters before entering the engine
- Hardware is described by capability profiles (H100, H200, B200), not hardcoded logic

## Milestone scope (M1–M8)

| Milestone | Scope |
|-----------|-------|
| M1 | Whole-GPU placement, FIFO scheduler, YAML configs, metrics JSON |
| M2 | Forge CRD bundle ingest |
| M3 | Scheduler trace replay + diff |
| M4 | MIG slice partition/reconfig delay |
| M5 | NVLink-domain-aware placement + `topology_penalties` |
| M6 | Quotas, priority, preemption, node-aware gang placement |
| M7 | Stepped RL session + Gymnasium env + PPO baseline |
| M8 | Jobs timeline export + Gantt/heatmap viz |
