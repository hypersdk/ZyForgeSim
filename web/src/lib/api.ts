import type {
  ClusterSnapshot,
  CompareResult,
  ConfigEntry,
  JobsTimeline,
  RunDetail,
  RunSummary,
  SchedulerDecision,
  SimulationMetrics,
} from "@/types/simulation";

const API = "/api";

function redirectToLogin() {
  if (typeof window !== "undefined") {
    const next = encodeURIComponent(window.location.pathname);
    window.location.href = `/login?next=${next}`;
  }
}

async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API}${path}`, {
    ...init,
    credentials: "include",
  });
  if (res.status === 401) {
    redirectToLogin();
    throw new Error("unauthorized");
  }
  if (!res.ok) {
    let detail: string | undefined;
    try {
      const body = (await res.json()) as { detail?: string };
      detail = body.detail;
    } catch {
      /* non-JSON error body */
    }
    throw new Error(detail ?? `request failed: ${path} (${res.status})`);
  }
  return res.json() as Promise<T>;
}

export async function fetchConfigs(): Promise<ConfigEntry[]> {
  return apiFetch<ConfigEntry[]>("/configs");
}

export async function fetchRuns(): Promise<RunSummary[]> {
  return apiFetch<RunSummary[]>("/runs");
}

export async function startRun(config: string): Promise<{ id: string }> {
  return apiFetch<{ id: string }>("/runs", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ config }),
  });
}

export async function fetchRun(id: string): Promise<RunDetail> {
  return apiFetch<RunDetail>(`/runs/${id}`);
}

export async function fetchTimeline(id: string): Promise<JobsTimeline> {
  return apiFetch<JobsTimeline>(`/runs/${id}/timeline`);
}

export async function fetchEvents(id: string): Promise<SchedulerDecision[]> {
  return apiFetch<SchedulerDecision[]>(`/runs/${id}/events`);
}

export async function fetchSnapshots(id: string): Promise<ClusterSnapshot[]> {
  return apiFetch<ClusterSnapshot[]>(`/runs/${id}/snapshots`);
}

export async function compareConfigs(configs: string[]): Promise<{ results: CompareResult[] }> {
  return apiFetch("/compare", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ configs }),
  });
}

export function pollRun(id: string, onUpdate: (run: RunDetail) => void, intervalMs = 1000): () => void {
  let active = true;
  const tick = async () => {
    while (active) {
      try {
        const run = await fetchRun(id);
        onUpdate(run);
        if (run.status === "completed" || run.status === "failed") break;
      } catch (e) {
        const message = e instanceof Error ? e.message : "";
        if (message === "unauthorized" || message.includes("(404)")) break;
      }
      await new Promise((r) => setTimeout(r, intervalMs));
    }
  };
  tick();
  return () => {
    active = false;
  };
}
