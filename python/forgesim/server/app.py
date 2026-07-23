"""FastAPI backend for ForgeSim web dashboard (Phase 2)."""

from __future__ import annotations

import asyncio
import json
import uuid
from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum
from pathlib import Path
from typing import Any

from fastapi import FastAPI, HTTPException, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field

REPO_ROOT = Path(__file__).resolve().parents[3]
CONFIG_DIR = REPO_ROOT / "configs" / "clusters"
RUNS_DIR = REPO_ROOT / "outputs" / "runs"


class RunStatus(str, Enum):
    pending = "pending"
    running = "running"
    completed = "completed"
    failed = "failed"


class StartRunRequest(BaseModel):
    config: str = Field(..., description="Cluster config filename under configs/clusters/")
    scheduler: str | None = Field(None, description="Override scheduler type")


@dataclass
class RunRecord:
    id: str
    config: str
    scheduler: str | None
    status: RunStatus = RunStatus.pending
    created_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    finished_at: str | None = None
    error: str | None = None
    metrics: dict[str, Any] | None = None
    timeline: dict[str, Any] | None = None
    decisions: list[dict[str, Any]] = field(default_factory=list)
    snapshots: list[dict[str, Any]] = field(default_factory=list)


RUNS: dict[str, RunRecord] = {}


def _list_configs() -> list[dict[str, str]]:
    if not CONFIG_DIR.exists():
        return []
    return [
        {"id": path.name, "path": str(path.relative_to(REPO_ROOT))}
        for path in sorted(CONFIG_DIR.glob("*.yaml"))
    ]


def _run_simulation_sync(config_name: str) -> dict[str, Any]:
    from forgesim import SimSession, _forgesim

    config_path = CONFIG_DIR / config_name
    if not config_path.exists():
        raise FileNotFoundError(f"config not found: {config_name}")

    report = _forgesim.run_report_from_config(str(config_path))
    metrics = json.loads(report["metrics"].to_json())
    timeline = json.loads(report["timeline"])
    decisions = list(report["decisions"])

    session = SimSession(str(config_path))
    snapshots: list[dict[str, Any]] = [session.reset()]
    steps = 0
    while not session.is_done and steps < 50_000:
        result = session.step_fifo()
        snapshots.append(result["observation"])
        steps += 1

    return {
        "metrics": metrics,
        "timeline": timeline,
        "decisions": decisions,
        "snapshots": snapshots,
    }


app = FastAPI(title="ForgeSim API", version="0.1.0")
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/api/health")
def health() -> dict[str, str]:
    return {"status": "ok"}


@app.get("/api/configs")
def list_configs() -> list[dict[str, str]]:
    return _list_configs()


@app.get("/api/runs")
def list_runs() -> list[dict[str, Any]]:
    return [
        {
            "id": run.id,
            "config": run.config,
            "scheduler": run.scheduler,
            "status": run.status.value,
            "created_at": run.created_at,
            "finished_at": run.finished_at,
        }
        for run in sorted(RUNS.values(), key=lambda r: r.created_at, reverse=True)
    ]


@app.post("/api/runs")
async def start_run(body: StartRunRequest) -> dict[str, str]:
    if not (CONFIG_DIR / body.config).exists():
        raise HTTPException(status_code=404, detail=f"unknown config: {body.config}")
    run_id = str(uuid.uuid4())
    RUNS[run_id] = RunRecord(id=run_id, config=body.config, scheduler=body.scheduler)
    asyncio.create_task(_execute_run(run_id))
    return {"id": run_id}


async def _execute_run(run_id: str) -> None:
    run = RUNS[run_id]
    run.status = RunStatus.running
    run_dir = RUNS_DIR / run_id
    run_dir.mkdir(parents=True, exist_ok=True)
    try:
        result = await asyncio.to_thread(_run_simulation_sync, run.config)
        run.metrics = result["metrics"]
        run.snapshots = result["snapshots"]
        run.decisions = result["decisions"]
        run.timeline = result["timeline"]
        run.status = RunStatus.completed
        run.finished_at = datetime.now(timezone.utc).isoformat()
        (run_dir / "metrics.json").write_text(json.dumps(run.metrics, indent=2))
        (run_dir / "timeline.json").write_text(json.dumps(run.timeline, indent=2))
        (run_dir / "decisions.json").write_text(json.dumps(run.decisions, indent=2))
    except Exception as exc:  # noqa: BLE001 — surface run failures to API clients
        run.status = RunStatus.failed
        run.error = str(exc)
        run.finished_at = datetime.now(timezone.utc).isoformat()


@app.get("/api/runs/{run_id}")
def get_run(run_id: str) -> dict[str, Any]:
    run = RUNS.get(run_id)
    if run is None:
        raise HTTPException(status_code=404, detail="run not found")
    return {
        "id": run.id,
        "config": run.config,
        "scheduler": run.scheduler,
        "status": run.status.value,
        "created_at": run.created_at,
        "finished_at": run.finished_at,
        "error": run.error,
        "metrics": run.metrics,
        "timeline": run.timeline,
        "decision_count": len(run.decisions),
    }


@app.get("/api/runs/{run_id}/snapshots")
def get_snapshots(run_id: str) -> list[dict[str, Any]]:
    run = RUNS.get(run_id)
    if run is None:
        raise HTTPException(status_code=404, detail="run not found")
    return run.snapshots


@app.get("/api/runs/{run_id}/timeline")
def get_timeline(run_id: str) -> dict[str, Any]:
    run = RUNS.get(run_id)
    if run is None:
        raise HTTPException(status_code=404, detail="run not found")
    if run.timeline is None:
        raise HTTPException(status_code=404, detail="timeline not ready")
    return run.timeline


@app.get("/api/runs/{run_id}/events")
def get_events(run_id: str) -> list[dict[str, Any]]:
    run = RUNS.get(run_id)
    if run is None:
        raise HTTPException(status_code=404, detail="run not found")
    return run.decisions


class CompareRequest(BaseModel):
    configs: list[str] = Field(..., min_length=2, description="Config filenames to compare")


@app.post("/api/compare")
async def compare_schedulers(body: CompareRequest) -> dict[str, Any]:
    results = []
    for config in body.configs:
        if not (CONFIG_DIR / config).exists():
            raise HTTPException(status_code=404, detail=f"unknown config: {config}")
        run_id = str(uuid.uuid4())
        RUNS[run_id] = RunRecord(id=run_id, config=config, scheduler=None)
        await _execute_run(run_id)
        run = RUNS[run_id]
        results.append(
            {
                "config": config,
                "status": run.status.value,
                "metrics": run.metrics,
                "run_id": run_id,
            }
        )
    return {"results": results}


@app.websocket("/ws/runs/{run_id}")
async def ws_run(websocket: WebSocket, run_id: str) -> None:
    run = RUNS.get(run_id)
    if run is None:
        await websocket.close(code=4404)
        return
    await websocket.accept()
    try:
        if run.snapshots:
            for snap in run.snapshots:
                await websocket.send_json({"type": "snapshot", "data": snap})
                await asyncio.sleep(0.05)
        while run.status == RunStatus.running:
            await asyncio.sleep(0.2)
        await websocket.send_json(
            {
                "type": "complete",
                "metrics": run.metrics,
                "decisions": run.decisions,
            }
        )
    except WebSocketDisconnect:
        return
