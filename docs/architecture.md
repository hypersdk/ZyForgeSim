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
2. Scheduler selects waiting jobs and allocates GPUs (all-or-nothing); a
   preemptive scheduler may also evict lower-priority running jobs back
   into the waiting queue to make room
3. `JobComplete` events free resources and trigger re-scheduling — each
   carries the `Job::run_generation` it completes, so a stale event from a
   run that was preempted before finishing is ignored rather than
   corrupting the job's later, actual completion
4. Clock advances only to the next event (no polling)

## Design invariants

- The Rust core never depends on Python or Gymnasium
- Schedulers share a common `Scheduler` trait for benchmarking
- Forge CRDs and traces convert to internal models via adapters before entering the engine
- Hardware is described by capability profiles (H100, H200, B200), not hardcoded logic

## Milestone 1 scope

Whole-GPU placement, FIFO scheduler, YAML configs, metrics JSON output.

Tenant quotas, priority scheduling, and preemption landed in M6. Future milestones add topology graphs (M5), Forge gang plugin parity, and RL wrappers (M7).
