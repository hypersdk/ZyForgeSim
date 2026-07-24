"use client";

import { useEffect, useState } from "react";
import { MetricsDashboard } from "@/components/MetricsCharts";
import { AppLink, Button, Card, FormField, PageHero, Select } from "@/components/ui";
import { fetchBenchmarkPresets, fetchBenchmarkReports, runBenchmark } from "@/lib/api";
import type { SchedulerBenchmarkReport, SimulationMetrics } from "@/types/simulation";

export default function BenchmarkPage() {
  const [configs, setConfigs] = useState<Array<{ id: string; path: string }>>([]);
  const [presets, setPresets] = useState<Array<{ id: string; description: string }>>([]);
  const [selected, setSelected] = useState("inference_llama.yaml");
  const [scheduler, setScheduler] = useState("fifo");
  const [metrics, setMetrics] = useState<SimulationMetrics | null>(null);
  const [benchmark, setBenchmark] = useState<SchedulerBenchmarkReport | null>(null);
  const [reports, setReports] = useState<Array<{ run_id: string; config: string; scheduler: string | null; benchmark: SchedulerBenchmarkReport | null }>>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchBenchmarkPresets()
      .then((data) => {
        setConfigs(data.configs);
        setPresets(data.workload_presets);
        if (data.configs.length && !selected) setSelected(data.configs[0].id);
      })
      .catch((e) => setError(e instanceof Error ? e.message : "Failed to load presets"));
    fetchBenchmarkReports().then(setReports).catch(console.error);
  }, [selected]);

  async function handleRun() {
    setBusy(true);
    setError(null);
    try {
      const result = await runBenchmark(selected, scheduler);
      setMetrics(result.metrics);
      setBenchmark(result.benchmark);
      const latest = await fetchBenchmarkReports();
      setReports(latest);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Benchmark run failed");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="space-y-6">
      <PageHero
        title="Benchmark Hub"
        subtitle="Scheduler benchmarks with inference TTFT/TPS metrics and score vectors."
        actions={<AppLink href="/">Dashboard</AppLink>}
      />
      <Card title="Run benchmark">
        <div className="grid gap-4 md:grid-cols-3">
          <FormField label="Config">
            <Select value={selected} onChange={(e) => setSelected(e.target.value)}>
              {configs.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.id}
                </option>
              ))}
            </Select>
          </FormField>
          <FormField label="Scheduler">
            <Select value={scheduler} onChange={(e) => setScheduler(e.target.value)}>
              <option value="fifo">fifo</option>
              <option value="priority">priority</option>
              <option value="preemptive">preemptive</option>
              <option value="forge">forge</option>
              <option value="bestfit">bestfit</option>
            </Select>
          </FormField>
          <div className="flex items-end">
            <Button onClick={handleRun} disabled={busy}>
              {busy ? "Running…" : "Run benchmark"}
            </Button>
          </div>
        </div>
        {error ? <p className="mt-3 text-sm text-red-400">{error}</p> : null}
        <div className="mt-4 text-sm text-hs-muted">
          Workload presets: {presets.map((p) => p.id).join(", ")}
        </div>
      </Card>
      <MetricsDashboard metrics={metrics} />
      {benchmark ? (
        <Card title="Score vector">
          <pre className="overflow-auto text-xs">{JSON.stringify(benchmark.score_vector, null, 2)}</pre>
          <p className="mt-2 text-sm text-hs-muted">
            Cost estimate: ${benchmark.cost_usd.toFixed(2)} · Jain fairness: {benchmark.jain_fairness.toFixed(3)}
          </p>
        </Card>
      ) : null}
      <Card title="Recent benchmark reports">
        <ul className="space-y-2 text-sm">
          {reports.map((r) => (
            <li key={r.run_id}>
              <AppLink href={`/runs/${r.run_id}`}>{r.config}</AppLink> · {r.scheduler ?? "default"}
            </li>
          ))}
        </ul>
      </Card>
    </div>
  );
}
