import type { SimulationMetrics } from "@/types/simulation";
import { chartColors } from "@/lib/theme";
import {
  Bar,
  BarChart,
  CartesianGrid,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { Card, MetricTile } from "./ui";

function cumulativeWait(metrics: SimulationMetrics): number {
  return metrics.mean_cumulative_wait_time ?? metrics.mean_wait_time;
}

export function MetricsDashboard({ metrics }: { metrics: SimulationMetrics | null }) {
  if (!metrics) return <Card title="Metrics">Run not complete.</Card>;

  const meanWait = cumulativeWait(metrics);
  const unschedulable = metrics.jobs_unschedulable ?? 0;

  const bars = [
    { name: "GPU util", value: metrics.gpu_utilization * 100 },
    { name: "Jobs done", value: (metrics.jobs_completed / Math.max(metrics.jobs_total, 1)) * 100 },
    { name: "Preemptions", value: metrics.preemptions },
    { name: "Failed", value: metrics.jobs_failed },
    { name: "Topo penalties", value: metrics.topology_penalties },
  ];

  const series = [
    { name: "makespan", value: metrics.makespan },
    { name: "mean wait", value: meanWait },
    { name: "queue max", value: metrics.queue_max_length },
    { name: "unschedulable", value: unschedulable },
    { name: "inflation", value: metrics.topology_runtime_inflation },
  ];

  return (
    <div className="space-y-4">
      <div className="metrics-grid">
        <MetricTile label="Makespan" value={`${metrics.makespan.toFixed(1)}s`} />
        <MetricTile label="GPU Utilization" value={`${(metrics.gpu_utilization * 100).toFixed(1)}%`} />
        <MetricTile label="Mean Cumulative Wait" value={`${meanWait.toFixed(2)}s`} />
        <MetricTile label="Jobs" value={`${metrics.jobs_completed}/${metrics.jobs_total}`} />
        <MetricTile label="Preemptions" value={String(metrics.preemptions)} />
        <MetricTile label="Failed Jobs" value={String(metrics.jobs_failed)} />
        <MetricTile label="Queue Max" value={String(metrics.queue_max_length)} />
        <MetricTile label="Unschedulable" value={String(unschedulable)} />
      </div>
      <Card title="Metrics Charts">
        <div className="grid gap-4 md:grid-cols-2">
          <ResponsiveContainer width="100%" height={200}>
            <BarChart data={bars}>
              <CartesianGrid strokeDasharray="3 3" stroke={chartColors.grid} />
              <XAxis dataKey="name" tick={{ fill: chartColors.tick, fontSize: 11 }} />
              <YAxis tick={{ fill: chartColors.tick, fontSize: 11 }} />
              <Tooltip
                contentStyle={{
                  backgroundColor: "#0b0f14",
                  border: "1px solid rgba(255,255,255,0.06)",
                  borderRadius: "8px",
                }}
              />
              <Bar dataKey="value" fill={chartColors.bar} />
            </BarChart>
          </ResponsiveContainer>
          <ResponsiveContainer width="100%" height={200}>
            <LineChart data={series}>
              <CartesianGrid strokeDasharray="3 3" stroke={chartColors.grid} />
              <XAxis dataKey="name" tick={{ fill: chartColors.tick, fontSize: 11 }} />
              <YAxis tick={{ fill: chartColors.tick, fontSize: 11 }} />
              <Tooltip
                contentStyle={{
                  backgroundColor: "#0b0f14",
                  border: "1px solid rgba(255,255,255,0.06)",
                  borderRadius: "8px",
                }}
              />
              <Line type="monotone" dataKey="value" stroke={chartColors.line} strokeWidth={2} />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </Card>
    </div>
  );
}
