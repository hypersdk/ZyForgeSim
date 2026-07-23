import type {
  ClusterSnapshot,
  ConfigEntry,
  JobsTimeline,
  RunDetail,
  RunSummary,
  SchedulerDecision,
  SimulationMetrics,
} from "@/types/simulation";

const API = "/api";

export async function fetchConfigs(): Promise<ConfigEntry[]> {
  const res = await fetch(`${API}/configs`);
  if (!res.ok) throw new Error("failed to load configs");
  return res.json();
}

export async function fetchRuns(): Promise<RunSummary[]> {
  const res = await fetch(`${API}/runs`);
  if (!res.ok) throw new Error("failed to load runs");
  return res.json();
}

export async function startRun(config: string): Promise<{ id: string }> {
  const res = await fetch(`${API}/runs`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ config }),
  });
  if (!res.ok) throw new Error("failed to start run");
  return res.json();
}

export async function fetchRun(id: string): Promise<RunDetail> {
  const res = await fetch(`${API}/runs/${id}`);
  if (!res.ok) throw new Error("run not found");
  return res.json();
}

export async function fetchTimeline(id: string): Promise<JobsTimeline> {
  const res = await fetch(`${API}/runs/${id}/timeline`);
  if (!res.ok) throw new Error("timeline not ready");
  return res.json();
}

export async function fetchEvents(id: string): Promise<SchedulerDecision[]> {
  const res = await fetch(`${API}/runs/${id}/events`);
  if (!res.ok) throw new Error("events not ready");
  return res.json();
}

export async function fetchSnapshots(id: string): Promise<ClusterSnapshot[]> {
  const res = await fetch(`${API}/runs/${id}/snapshots`);
  if (!res.ok) throw new Error("snapshots not ready");
  return res.json();
}

export async function compareConfigs(configs: string[]): Promise<{
  results: Array<{ config: string; status: string; metrics: SimulationMetrics | null; run_id: string }>;
}> {
  const res = await fetch(`${API}/compare`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ configs }),
  });
  if (!res.ok) throw new Error("compare failed");
  return res.json();
}

export function pollRun(id: string, onUpdate: (run: RunDetail) => void, intervalMs = 1000): () => void {
  let active = true;
  const tick = async () => {
    while (active) {
      try {
        const run = await fetchRun(id);
        onUpdate(run);
        if (run.status === "completed" || run.status === "failed") break;
      } catch {
        /* retry */
      }
      await new Promise((r) => setTimeout(r, intervalMs));
    }
  };
  tick();
  return () => {
    active = false;
  };
}
