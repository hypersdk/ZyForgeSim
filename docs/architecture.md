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
  ├── forgesim-core      Event engine, cluster, resources
  ├── forgesim-scheduler Scheduling policies
  ├── forgesim-config    YAML loading
  ├── forgesim-metrics   Makespan, wait, utilization
  ├── forgesim-cli       forge-sim binary
  └── forgesim-py        Python bindings
```

## Simulation loop

1. Jobs arrive via `JobArrival` events
2. Scheduler selects waiting jobs and allocates GPUs (all-or-nothing)
3. `JobComplete` events free resources and trigger re-scheduling
4. Clock advances only to the next event (no polling)

## Design invariants

- The Rust core never depends on Python or Gymnasium
- Schedulers share a common `Scheduler` trait for benchmarking
- Forge CRDs and traces convert to internal models via adapters before entering the engine
- Hardware is described by capability profiles (H100, H200, B200), not hardcoded logic

## Milestone 1 scope

Whole-GPU placement, FIFO scheduler, YAML configs, metrics JSON output.

Future milestones add topology graphs, tenant quotas, gang scheduling policy, preemption, and RL wrappers.
