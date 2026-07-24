export interface SimulationMetrics {
  makespan: number;
  mean_wait_time: number;
  mean_cumulative_wait_time?: number;
  gpu_utilization: number;
  jobs_completed: number;
  jobs_total: number;
  queue_max_length: number;
  jobs_unschedulable?: number;
  mig_reconfigs: number;
  preemptions: number;
  topology_penalties: number;
  topology_runtime_inflation: number;
  jobs_failed: number;
}

export interface JobTimelineRecord {
  job_id: string;
  name: string;
  arrival_time: number;
  start_time: number | null;
  finish_time: number | null;
  runtime: number;
  gpu_count: number;
  assigned_gpus: string[];
  priority: number;
  tenant: string | null;
  state: string;
}

export interface JobsTimeline {
  makespan: number;
  gpu_count: number;
  jobs: JobTimelineRecord[];
}

export interface SchedulerDecision {
  time: number;
  kind: string;
  job_id: string | null;
  job_name: string | null;
  gpu_ids: string[];
  message: string;
}

export interface GpuSnapshot {
  id: string;
  node_id: string;
  busy: boolean;
  utilization: number;
  job_id: string | null;
  job_name: string | null;
  nvlink_group: number | null;
}

export interface ClusterSnapshot {
  clock: number;
  free_gpus: number;
  waiting: number;
  running: number;
  finished: number;
  node_count: number;
  gpu_count: number;
  queue_jobs: Array<{ id: string; name: string; priority: number; tenant: string | null; gpu_count: number; state: string }>;
  nodes: Array<{ id: string; gpus: GpuSnapshot[] }>;
}

export interface RunSummary {
  id: string;
  config: string;
  scheduler: string | null;
  status: string;
  created_at: string;
  finished_at: string | null;
}

export interface RunDetail extends RunSummary {
  error: string | null;
  metrics: SimulationMetrics | null;
  timeline: JobsTimeline | null;
  decision_count: number;
}

export interface ConfigEntry {
  id: string;
  path: string;
}

export interface CompareResult {
  config: string;
  status: string;
  metrics: SimulationMetrics | null;
  run_id: string;
}
