"use client";

import type { CompareResult, SimulationMetrics } from "@/types/simulation";
import { AppLink, MetricTile, StatusBadge } from "./ui";

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
  {
    label: "Mean Cumulative Wait",
    format: (m) => `${cumulativeWait(m).toFixed(2)}s`,
    lowerIsBetter: true,
  },
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
  { label: "TTFT p50", format: (m) => `${(m.ttft_p50 ?? 0).toFixed(3)}s`, lowerIsBetter: true },
  { label: "TTFT p99", format: (m) => `${(m.ttft_p99 ?? 0).toFixed(3)}s`, lowerIsBetter: true },
  { label: "TPS mean", format: (m) => (m.tps_mean ?? 0).toFixed(1), lowerIsBetter: false },
  { label: "Goodput", format: (m) => `${((m.goodput ?? 0) * 100).toFixed(1)}%`, lowerIsBetter: false },
  { label: "Queue delay p99", format: (m) => `${(m.queue_delay_p99 ?? 0).toFixed(3)}s`, lowerIsBetter: true },
];

function metricValue(m: SimulationMetrics, label: string): number {
  switch (label) {
    case "Makespan":
      return m.makespan;
    case "GPU Utilization":
      return m.gpu_utilization;
    case "Mean Cumulative Wait":
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
    case "TTFT p50":
      return m.ttft_p50 ?? 0;
    case "TTFT p99":
      return m.ttft_p99 ?? 0;
    case "TPS mean":
      return m.tps_mean ?? 0;
    case "Goodput":
      return m.goodput ?? 0;
    case "Queue delay p99":
      return m.queue_delay_p99 ?? 0;
    default:
      return 0;
  }
}

function formatDelta(fieldLabel: string, baseline: number, value: number, lowerIsBetter: boolean): string | null {
  const tie = Math.abs(baseline - value) < 1e-6;
  if (tie) return "Tie";
  const better = lowerIsBetter ? value < baseline : value > baseline;
  if (fieldLabel === "GPU Utilization" || fieldLabel === "Jobs Completed") {
    const pct = Math.abs((value - baseline) * 100);
    return better ? `Better (Δ ${pct.toFixed(1)}%)` : `Worse (Δ ${pct.toFixed(1)}%)`;
  }
  const delta = Math.abs(value - baseline);
  const unit = fieldLabel === "Makespan" || fieldLabel === "Mean Cumulative Wait" ? "s" : "";
  return better ? `Better (Δ ${delta.toFixed(delta < 10 ? 2 : 1)}${unit})` : `Worse (Δ ${delta.toFixed(delta < 10 ? 2 : 1)}${unit})`;
}

export function ComparePanel({ results }: { results: CompareResult[] }) {
  if (!results.length) return null;

  const baseline = results[0]?.metrics;

  return (
    <div className="compare-grid mt-4">
      {results.map((r, resultIndex) => (
        <div key={`${r.config}-${r.run_id}`} className="compare-result-card">
          <div className="compare-result-header">
            <div>
              <div className="text-sm font-semibold text-hs-heading">{r.config}</div>
              <div className="mt-1 flex flex-wrap items-center gap-2">
                <StatusBadge status={r.status} />
                <span className="font-mono text-xs text-hs-muted">{r.run_id.slice(0, 8)}</span>
              </div>
            </div>
            {r.status === "completed" ? (
              <AppLink href={`/runs/${r.run_id}`}>View run</AppLink>
            ) : null}
          </div>
          {r.metrics ? (
            <div className="compare-metrics-grid">
              {COMPARE_FIELDS.map((field) => {
                const value = field.format(r.metrics!);
                let highlight = "";
                let deltaText: string | null = null;
                if (baseline && results.length === 2 && field.lowerIsBetter != null && resultIndex > 0) {
                  const a = metricValue(baseline, field.label);
                  const b = metricValue(r.metrics!, field.label);
                  const better = field.lowerIsBetter ? b < a : b > a;
                  const tie = Math.abs(a - b) < 1e-6;
                  deltaText = formatDelta(field.label, a, b, field.lowerIsBetter);
                  if (!tie && better) highlight = "compare-metric-better";
                  if (!tie && !better) highlight = "compare-metric-worse";
                }
                return (
                  <div key={field.label} className={highlight}>
                    <MetricTile label={field.label} value={value} />
                    {deltaText ? <p className="compare-metric-delta">{deltaText}</p> : null}
                  </div>
                );
              })}
            </div>
          ) : (
            <p className="text-sm text-hs-muted">
              {r.status === "failed"
                ? "Simulation failed — no metrics available for this config."
                : "No metrics available for this run."}
            </p>
          )}
        </div>
      ))}
    </div>
  );
}
