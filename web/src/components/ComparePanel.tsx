"use client";

import type { SimulationMetrics } from "@/types/simulation";
import { MetricTile } from "./ui";

export function ComparePanel({
  results,
}: {
  results: Array<{ config: string; metrics: SimulationMetrics | null }>;
}) {
  if (!results.length) return null;

  return (
    <div className="compare-grid mt-4">
      {results.map((r) => (
        <div key={r.config} className="compare-result-card">
          <div className="mb-3 text-sm font-semibold text-hs-heading">{r.config}</div>
          {r.metrics ? (
            <div className="grid grid-cols-2 gap-2">
              <MetricTile label="Makespan" value={`${r.metrics.makespan.toFixed(1)}s`} />
              <MetricTile label="Utilization" value={`${(r.metrics.gpu_utilization * 100).toFixed(1)}%`} />
            </div>
          ) : (
            <p className="text-sm text-hs-muted">No metrics available for this run.</p>
          )}
        </div>
      ))}
    </div>
  );
}
