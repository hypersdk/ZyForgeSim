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
   locality (with scatter fallback tracked as `topology_penalties`). Cross-domain
   placement inflates job runtime via `TopologyGraph` (`topology_runtime_inflation`).
4. Gang jobs with `gang_timeout_secs` schedule a `GangTimeout` event; jobs still
   waiting when it fires move to `JobState::Failed` (`jobs_failed` metric).
5. `JobComplete` events free resources and trigger re-scheduling — each
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
(finished, running, and waiting jobs) and a `decisions` log for replay.
The CLI writes timeline via `--jobs-output`; Python `forgesim.viz` renders
Gantt charts and GPU utilization heatmaps.

## UI stack (staged)

The Rust core never knows about the UI — it exposes APIs and events only.

```
ForgeSim Core (Rust)
        │
Python Bindings (PyO3: SimSession, SimResult, run_report_from_config)
        │
   ┌────┴────┐
   ▼         ▼
Rich CLI   FastAPI + WebSockets
dashboard      │
           Next.js dashboard
```

| Phase | Deliverable | Location |
|-------|-------------|----------|
| 1 | Rich live terminal dashboard | `python/forgesim/dashboard/` |
| 2 | FastAPI run registry + replay API | `python/forgesim/server/` |
| 2 | Next.js monitor (Gantt, topology, compare) | `web/` |

See [docs/ui_roadmap.md](ui_roadmap.md) for the full roadmap including Zyvor Forge integration.

**User guide:** [docs/ui_dashboard.md](ui_dashboard.md) — setup scripts, CLI dashboard, web dashboard, API reference, troubleshooting.

## Benchmark platform (planned)

ForgeSim is extending from scheduler simulation (M1–M8) into a three-layer **benchmark platform** that connects scheduling decisions to LLM serving metrics (TTFT, TPS, goodput), calibrated via AIPerf.

```text
Simulation Layer (Rust DES)  →  Benchmark Layer (traces, AIPerf, OpenAI shim)  →  Analytics Layer (dashboard, twin, CI)
```

**Roadmap:** [docs/benchmark_platform.md](benchmark_platform.md) — phased plan P0–P10, UI/tests per phase, multi-model review synthesis.

| Phase | Focus |
|-------|-------|
| P0 | Simulation + web replay hardening |
| P1 | Inference performance model (gate for TTFT/TPS) |
| P2–P3 | Synthetic LLM workloads + serving trace I/O |
| P4–P5 | Scheduler benchmark score + dashboard |
| P6–P7 | OpenAI shim + AIPerf calibration |
| P8–P10 | What-if, digital twin, CI regression gates |

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
| M5 | NVLink-domain placement, `topology_penalties`, runtime inflation |
| M6 | Quotas, priority, preemption, gang spread + timeout, `ForgeScheduler`, `BestFitScheduler` |
| M7 | Stepped RL session + Gymnasium env + PPO baseline |
| M8 | Jobs timeline export + Gantt/heatmap viz |
| **P0–P10** | **Benchmark platform** — inference model, AIPerf, twin, CI ([benchmark_platform.md](benchmark_platform.md)) |
