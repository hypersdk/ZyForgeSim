import type { SimulationMetrics } from "@/types/simulation";
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

export function MetricsDashboard({ metrics }: { metrics: SimulationMetrics | null }) {
  if (!metrics) return <Card title="Metrics">Run not complete.</Card>;

  const bars = [
    { name: "GPU util", value: metrics.gpu_utilization * 100 },
    { name: "Jobs done", value: (metrics.jobs_completed / Math.max(metrics.jobs_total, 1)) * 100 },
    { name: "Preemptions", value: metrics.preemptions },
    { name: "Topo penalties", value: metrics.topology_penalties },
  ];

  const series = [
    { name: "makespan", value: metrics.makespan },
    { name: "mean wait", value: metrics.mean_wait_time },
    { name: "inflation", value: metrics.topology_runtime_inflation },
    { name: "failed", value: metrics.jobs_failed },
  ];

  return (
    <div className="space-y-4">
      <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
        <MetricTile label="Makespan" value={`${metrics.makespan.toFixed(1)}s`} />
        <MetricTile label="GPU Utilization" value={`${(metrics.gpu_utilization * 100).toFixed(1)}%`} />
        <MetricTile label="Mean Wait" value={`${metrics.mean_wait_time.toFixed(2)}s`} />
        <MetricTile label="Jobs" value={`${metrics.jobs_completed}/${metrics.jobs_total}`} />
      </div>
      <Card title="Metrics">
        <div className="grid gap-4 md:grid-cols-2">
          <ResponsiveContainer width="100%" height={180}>
            <BarChart data={bars}>
              <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
              <XAxis dataKey="name" tick={{ fill: "#94a3b8", fontSize: 11 }} />
              <YAxis tick={{ fill: "#94a3b8", fontSize: 11 }} />
              <Tooltip />
              <Bar dataKey="value" fill="#38bdf8" />
            </BarChart>
          </ResponsiveContainer>
          <ResponsiveContainer width="100%" height={180}>
            <LineChart data={series}>
              <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
              <XAxis dataKey="name" tick={{ fill: "#94a3b8", fontSize: 11 }} />
              <YAxis tick={{ fill: "#94a3b8", fontSize: 11 }} />
              <Tooltip />
              <Line type="monotone" dataKey="value" stroke="#2dd4bf" strokeWidth={2} />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </Card>
    </div>
  );
}
