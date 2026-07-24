"use client";

import type { SimulationMetrics } from "@/types/simulation";
import { MetricTile } from "./ui";

function cumulativeWait(metrics: SimulationMetrics): number {
  return metrics.mean_cumulative_wait_time ?? metrics.mean_wait_time;
}

const COMPARE_FIELDS: Array<{
  label: string;
  format: (m: SimulationMetrics) => string;
  lowerIsBetter?: boolean;
}> = [
  { label: "Makespan", format: (m) => `${m.makespan.toFixed(1)}s`, lowerIsBetter: true },
  { label: "GPU Utilization", format: (m) => `${(m.gpu_utilization * 100).toFixed(1)}%`, lowerIsBetter: false },
  { label: "Mean Wait", format: (m) => `${cumulativeWait(m).toFixed(2)}s`, lowerIsBetter: true },
  {
    label: "Jobs Completed",
    format: (m) => `${m.jobs_completed}/${m.jobs_total}`,
    lowerIsBetter: false,
  },
  { label: "Preemptions", format: (m) => String(m.preemptions), lowerIsBetter: true },
  { label: "Failed", format: (m) => String(m.jobs_failed), lowerIsBetter: true },
  { label: "Queue Max", format: (m) => String(m.queue_max_length), lowerIsBetter: true },
  {
    label: "Unschedulable",
    format: (m) => String(m.jobs_unschedulable ?? 0),
    lowerIsBetter: true,
  },
  { label: "Topo Penalties", format: (m) => String(m.topology_penalties), lowerIsBetter: true },
];

function metricValue(m: SimulationMetrics, label: string): number {
  switch (label) {
    case "Makespan":
      return m.makespan;
    case "GPU Utilization":
      return m.gpu_utilization;
    case "Mean Wait":
      return cumulativeWait(m);
    case "Jobs Completed":
      return m.jobs_completed / Math.max(m.jobs_total, 1);
    case "Preemptions":
      return m.preemptions;
    case "Failed":
      return m.jobs_failed;
    case "Queue Max":
      return m.queue_max_length;
    case "Unschedulable":
      return m.jobs_unschedulable ?? 0;
    case "Topo Penalties":
      return m.topology_penalties;
    default:
      return 0;
  }
}

export function ComparePanel({
  results,
}: {
  results: Array<{ config: string; metrics: SimulationMetrics | null }>;
}) {
  if (!results.length) return null;

  const baseline = results[0]?.metrics;

  return (
    <div className="compare-grid mt-4">
      {results.map((r) => (
        <div key={r.config} className="compare-result-card">
          <div className="mb-3 text-sm font-semibold text-hs-heading">{r.config}</div>
          {r.metrics ? (
            <div className="compare-metrics-grid">
              {COMPARE_FIELDS.map((field) => {
                const value = field.format(r.metrics!);
                let highlight = "";
                if (baseline && results.length === 2 && field.lowerIsBetter != null) {
                  const a = metricValue(baseline, field.label);
                  const b = metricValue(r.metrics!, field.label);
                  const better = field.lowerIsBetter ? b < a : b > a;
                  const tie = Math.abs(a - b) < 1e-6;
                  if (!tie && better) highlight = "compare-metric-better";
                  if (!tie && !better && r.config !== results[0].config) highlight = "compare-metric-worse";
                }
                return (
                  <div key={field.label} className={highlight}>
                    <MetricTile label={field.label} value={value} />
                  </div>
                );
              })}
            </div>
          ) : (
            <p className="text-sm text-hs-muted">No metrics available for this run.</p>
          )}
        </div>
      ))}
    </div>
  );
}
