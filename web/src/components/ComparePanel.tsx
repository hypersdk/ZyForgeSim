"use client";

import type { SimulationMetrics } from "@/types/simulation";
import { Card, MetricTile } from "./ui";

export function ComparePanel({
  results,
}: {
  results: Array<{ config: string; metrics: SimulationMetrics | null }>;
}) {
  if (!results.length) return null;
  return (
    <Card title="Compare Schedulers / Configs">
      <div className="grid gap-4 md:grid-cols-2">
        {results.map((r) => (
          <div key={r.config} className="rounded-hs border border-hs-border bg-hs-bg/40 p-3">
            <div className="mb-2 font-medium text-hs-heading">{r.config}</div>
            {r.metrics ? (
              <div className="grid grid-cols-2 gap-2">
                <MetricTile label="Makespan" value={`${r.metrics.makespan.toFixed(1)}s`} />
                <MetricTile label="Utilization" value={`${(r.metrics.gpu_utilization * 100).toFixed(1)}%`} />
              </div>
            ) : (
              <div className="text-sm text-hs-muted">No metrics</div>
            )}
          </div>
        ))}
      </div>
    </Card>
  );
}
