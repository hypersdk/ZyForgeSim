# ForgeSim UI Dashboard Guide

ForgeSim provides two ways to monitor simulations: a **Rich terminal dashboard** (Phase 1) and a **web dashboard** (Phase 2). Both sit on top of the Rust core via PyO3 — the engine never depends on UI code.

See also: [UI roadmap](ui_roadmap.md) · [Architecture](architecture.md) · [Web app README](../web/README.md)

---

## Prerequisites

| Requirement | Notes |
|-------------|-------|
| Rust toolchain | `cargo` — builds the simulation core and PyO3 extension |
| Python ≥ 3.10 | Managed via `.venv` (do **not** use macOS system Python 3.9) |
| Node.js ≥ 18 | For the web UI only (`web/`) |
| `maturin` | Installed into `.venv` by setup script |

---

## One-time dev environment setup

From the repo root:

```bash
./scripts/setup_dev.sh
```

This script:

1. Creates `.venv` (uses `python -m venv --without-pip` to avoid broken Homebrew `ensurepip`)
2. Bootstraps pip via `get-pip.py` or `uv`
3. Installs `rich`, `pyyaml`, `maturin`
4. Builds the PyO3 extension (`forgesim._forgesim`)
5. Patches `.venv/bin/activate` with the Homebrew expat fix (macOS)

Activate the venv in new shells:

```bash
source .venv/bin/activate
```

### macOS Homebrew Python troubleshooting

Homebrew `python@3.13` can fail with:

```text
ImportError: ... pyexpat ... Symbol not found: _XML_SetAllocTrackerActivationThreshold
```

**Option A — automatic fix (built into scripts):**  
Setup and run scripts prepend Homebrew's expat library to `DYLD_LIBRARY_PATH`. Retry:

```bash
rm -rf .venv
./scripts/setup_dev.sh
```

**Option B — use uv (recommended if Option A fails):**

```bash
brew install uv
rm -rf .venv
USE_UV=1 ./scripts/setup_dev.sh
```

This downloads a standalone Python 3.12 via `uv`, avoiding Homebrew pyexpat entirely.

---

## Helper scripts reference

All scripts live in [`scripts/`](../scripts/). Shared logic is in [`scripts/common.sh`](../scripts/common.sh).

| Script | Purpose |
|--------|---------|
| [`setup_dev.sh`](../scripts/setup_dev.sh) | Create `.venv`, build Rust extension, install Python deps |
| [`run_live_dashboard.sh`](../scripts/run_live_dashboard.sh) | Rich terminal live dashboard |
| [`run_web_dashboard.sh`](../scripts/run_web_dashboard.sh) | Start FastAPI + Next.js together |
| [`run_web_api.sh`](../scripts/run_web_api.sh) | FastAPI backend only (port 8080) |
| [`run_web_ui.sh`](../scripts/run_web_ui.sh) | Next.js frontend only (port 3000) |

---

## Phase 1 — Rich CLI dashboard

Live terminal view: simulation time, running/queued jobs, per-GPU utilization bars, queue list.

### Run

```bash
./scripts/run_live_dashboard.sh --config configs/clusters/small_h100.yaml
```

Options (passed through to the Python entry point):

| Flag | Default | Description |
|------|---------|-------------|
| `--config` | `configs/clusters/small_h100.yaml` | Cluster config YAML |
| `--refresh-hz` | `4.0` | Display refresh rate |
| `--plain` | off | Plain text instead of Rich panels |

### Alternative entry points

```bash
source .venv/bin/activate
python python/examples/live_dashboard.py --config configs/clusters/small_h100.yaml
python -m forgesim.dashboard --config configs/clusters/small_h100.yaml
forge-sim-dashboard --config configs/clusters/small_h100.yaml   # after pip install -e .
```

### How it works

- Uses `SimSession` (stepped DES) with `step_fifo()` to auto-advance
- Reads extended `ClusterSnapshot` (nodes, GPUs, queue) from PyO3
- Renders via [`python/forgesim/dashboard/`](../python/forgesim/dashboard/)

---

## Phase 2 — Web dashboard

Browser UI: run simulations, view metrics, Gantt charts, GPU topology, scheduler replay, compare configs.

### One-time web setup

```bash
./scripts/setup_dev.sh          # if not already done
cd web && npm install && cd ..
```

### Run (recommended — one terminal)

```bash
./scripts/run_web_dashboard.sh
```

| URL | Service |
|-----|---------|
| http://localhost:3000 | Next.js UI |
| http://localhost:8080/api/health | FastAPI backend |

Press **Ctrl+C** to stop both servers.

### Run (two terminals)

Terminal 1 — API:

```bash
./scripts/run_web_api.sh
```

Terminal 2 — UI:

```bash
./scripts/run_web_ui.sh
```

### Environment variables

| Variable | Default | Used by |
|----------|---------|---------|
| `API_PORT` | `8080` | `run_web_api.sh`, `run_web_dashboard.sh` |
| `UI_PORT` | `3000` | `run_web_ui.sh`, `run_web_dashboard.sh` |
| `HOST` | `0.0.0.0` | `run_web_api.sh` only |
| `USE_UV` | unset | `setup_dev.sh` — set to `1` to force uv-managed Python |

Example:

```bash
API_PORT=9000 UI_PORT=3001 ./scripts/run_web_dashboard.sh
```

---

## Web dashboard features

### Home page (`/`)

- Pick a config from `configs/clusters/*.yaml`
- **Run simulation** — triggers async run via API
- Recent runs table with status
- **Compare two configs** side-by-side (makespan, utilization)

### Run detail page (`/runs/:id`)

| Panel | Description |
|-------|-------------|
| Cluster summary | Nodes, GPUs, running, queue counts |
| Replay controls | Play / pause / prev / next / speed (0.5×–10×) |
| Cluster view | Per-node GPU grid (idle / busy coloring) |
| MIG layout | Placeholder slice grid per GPU |
| Topology | React Flow NVLink/PCIe graph |
| Gantt | Wait (orange), run (teal), failed (red dashed) bars |
| Queue / jobs table | Priority, tenant, GPUs, state |
| Metrics | Makespan, utilization, charts |

---

## FastAPI backend

**Module:** [`python/forgesim/server/app.py`](../python/forgesim/server/app.py)

### REST endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/health` | Liveness check |
| `GET` | `/api/configs` | List `configs/clusters/*.yaml` |
| `GET` | `/api/runs` | List recent runs |
| `POST` | `/api/runs` | Start run — body: `{ "config": "small_h100.yaml" }` |
| `GET` | `/api/runs/:id` | Run status + metrics summary |
| `GET` | `/api/runs/:id/timeline` | Jobs timeline JSON |
| `GET` | `/api/runs/:id/events` | Scheduler decision log (replay) |
| `GET` | `/api/runs/:id/snapshots` | Stepped cluster snapshots |
| `POST` | `/api/compare` | Compare configs — body: `{ "configs": ["a.yaml", "b.yaml"] }` |

### WebSocket

| Path | Description |
|------|-------------|
| `WS /ws/runs/:id` | Stream snapshot frames, then `complete` message with metrics |

### Run artifacts

Completed runs are written under:

```text
outputs/runs/{uuid}/
  metrics.json
  timeline.json
  decisions.json
```

### Manual API start

```bash
source .venv/bin/activate
export PYTHONPATH=python
python -m uvicorn forgesim.server.app:app --reload --port 8080
```

---

## Next.js frontend

**Location:** [`web/`](../web/)

Proxies `/api/*` and `/ws/*` to `http://localhost:8080` via [`web/next.config.js`](../web/next.config.js).

### Manual UI start

```bash
cd web
npm run dev
```

### Production build

```bash
cd web
npm run build
npm run start
```

(Serve the API separately; configure proxy or set `NEXT_PUBLIC_API_URL` if you add one.)

---

## Architecture

```text
ForgeSim Core (Rust)
        │
Python Bindings (PyO3: SimSession, SimResult, run_report_from_config)
        │
   ┌────┴────────────────────┐
   ▼                         ▼
Rich CLI dashboard      FastAPI (REST + WebSocket)
python/forgesim/              │
/dashboard/                   ▼
                         Next.js (web/)
```

**Scheduler decisions** for replay are recorded in [`crates/forgesim-core/src/decision_log.rs`](../crates/forgesim-core/src/decision_log.rs) and exported in `SimulationReport.decisions`.

---

## Quick verification

After `./scripts/setup_dev.sh`:

```bash
source .venv/bin/activate
python -c "import forgesim._forgesim; import rich; print('OK')"
```

After `./scripts/run_web_api.sh`:

```bash
curl http://127.0.0.1:8080/api/health
curl http://127.0.0.1:8080/api/configs
```

After `./scripts/run_web_dashboard.sh`:

Open http://localhost:3000, select `small_h100.yaml`, click **Run simulation**.

---

## Common errors

| Error | Fix |
|-------|-----|
| `No module named 'forgesim'` | Run `./scripts/setup_dev.sh`; use `.venv/bin/python` or activate venv |
| `No module named pip` | `rm -rf .venv && ./scripts/setup_dev.sh` |
| `pyexpat` / `libexpat` symbol error | `USE_UV=1 ./scripts/setup_dev.sh` or `brew reinstall expat python@3.13` |
| `ForgeSim extension not built` | `./scripts/setup_dev.sh` (runs `maturin develop`) |
| UI shows no configs / API errors | Ensure `./scripts/run_web_api.sh` is running on port 8080 |
| `typer` version warning from `zyvor-qa-agent` | Harmless outside `.venv`; ignore unless installed in `.venv` |
