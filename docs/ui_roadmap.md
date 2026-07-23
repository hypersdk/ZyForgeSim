# ForgeSim UI Roadmap

ForgeSim's UI grows in stages on top of the Rust core and PyO3 bindings. The engine never depends on UI code.

## Architecture

```
ForgeSim Core (Rust)
        │
Python Bindings (PyO3)
        │
   ┌────┴────────────────┐
   ▼                     ▼
Rich CLI dashboard    FastAPI (REST + WebSocket)
                           │
                      Next.js web dashboard
```

## Phase 1 — Rich CLI dashboard (done)

- **Module:** `python/forgesim/dashboard/`
- **Entry:** `python/examples/live_dashboard.py` or `python -m forgesim.dashboard`
- **Deps:** `pip install -e '.[dashboard]'` (Rich)
- **Data:** `SimSession.step_fifo()` + extended `ClusterSnapshot` (per-GPU util, queue, nodes)

## Phase 2 — Web dashboard (done, MVP)

### Backend — FastAPI

- **Module:** `python/forgesim/server/app.py`
- **Deps:** `pip install -e '.[server]'`
- **Endpoints:** `/api/health`, `/api/configs`, `/api/runs`, `/api/compare`, `/ws/runs/{id}`
- **Artifacts:** `outputs/runs/{uuid}/` (`metrics.json`, `timeline.json`, `decisions.json`)

### Frontend — Next.js

- **Location:** `web/`
- **Stack:** Next.js, React, TypeScript, Tailwind, Recharts, React Flow, Zustand
- **Views:** cluster summary, GPU grid, topology graph, Gantt (incl. failed jobs), replay scrubber, metrics charts, config compare

### Engine support

- `SchedulerDecision` log in `crates/forgesim-core/src/decision_log.rs`
- Recorded on job arrival, schedule, complete, gang timeout
- Exported in `SimulationReport.decisions`

## Phase 3 — Zyvor Forge integration (future)

Same UI components, different adapters:

| Mode | Source |
|------|--------|
| Simulation | YAML / forge bundle → FastAPI |
| Replay | M3 trace JSONL → event stream |
| Live | Forge export → `ClusterSnapshot` mapping |

Long-term vision: **Grafana meets Kubernetes Dashboard meets DCGM — focused on AI scheduling** — see, replay, and compare scheduling decisions over time.
