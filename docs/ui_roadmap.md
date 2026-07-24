# ForgeSim UI Roadmap

ForgeSim's UI grows in stages on top of the Rust core and PyO3 bindings. The engine never depends on UI code.

**Full user guide:** [ui_dashboard.md](ui_dashboard.md)

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

## Scripts (quick reference)

| Script | Purpose |
|--------|---------|
| `./scripts/setup_dev.sh` | One-time `.venv` + Rust extension setup |
| `./scripts/run_live_dashboard.sh` | Rich terminal dashboard |
| `./scripts/run_web_dashboard.sh` | Web API + UI together |
| `./scripts/run_web_api.sh` | FastAPI only (:8080) |
| `./scripts/run_web_ui.sh` | Next.js only (:3000) |

## Phase 1 — Rich CLI dashboard (done)

- **Module:** `python/forgesim/dashboard/`
- **Run:** `./scripts/run_live_dashboard.sh --config configs/clusters/small_h100.yaml`
- **Data:** `SimSession.step_fifo()` + extended `ClusterSnapshot`

## Phase 2 — Web dashboard (done, MVP)

- **Backend:** `python/forgesim/server/app.py` — `./scripts/run_web_api.sh`
- **Frontend:** `web/` — `./scripts/run_web_ui.sh` or `./scripts/run_web_dashboard.sh`
- **Views:** cluster summary, replay, Gantt, topology, metrics, compare

## Phase 3 — Zyvor Forge integration (future)

| Mode | Source |
|------|--------|
| Simulation | YAML / forge bundle → FastAPI |
| Replay | M3 trace JSONL → event stream |
| Live | Forge export → `ClusterSnapshot` mapping |

Long-term vision: **Grafana meets Kubernetes Dashboard meets DCGM — focused on AI scheduling**.

## Phase 4 — Benchmark dashboard (planned)

Extends Phase 2 web UI — not a separate app. See [benchmark_platform.md](benchmark_platform.md).

| Route (planned) | Purpose |
|-----------------|---------|
| `/benchmark` | TTFT, TPS, goodput, sim vs measured (AIPerf) |
| `/what-if` | Cluster/scheduler sweep matrix |

Backend: inference metrics from P1, AIPerf import from P7, compare/score from P4.
