"use client";

import { useState } from "react";
import { AppLink, Button, Card, FormField, PageHero, Select } from "@/components/ui";
import { runWhatIf } from "@/lib/api";

export default function WhatIfPage() {
  const [baseConfig, setBaseConfig] = useState("inference_llama.yaml");
  const [rows, setRows] = useState<Array<Record<string, unknown>>>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSweep() {
    setBusy(true);
    setError(null);
    try {
      const data = await runWhatIf(baseConfig, ["fifo", "preemptive"]);
      setRows(data.results);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Sweep failed");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="space-y-6">
      <PageHero title="What-if Analysis" subtitle="Compare schedulers on the same inference workload." actions={<AppLink href="/benchmark">Benchmark</AppLink>} />
      <Card title="Sweep">
        <FormField label="Base config">
          <Select value={baseConfig} onChange={(e) => setBaseConfig(e.target.value)}>
            <option value="inference_llama.yaml">inference_llama.yaml</option>
            <option value="small_h100.yaml">small_h100.yaml</option>
          </Select>
        </FormField>
        <Button className="mt-4" onClick={handleSweep} disabled={busy}>
          {busy ? "Running sweep…" : "Run fifo vs preemptive"}
        </Button>
        {error ? <p className="mt-3 text-sm text-red-400">{error}</p> : null}
      </Card>
      <Card title="Results matrix">
        <div className="overflow-auto">
          <table className="min-w-full text-sm">
            <thead>
              <tr className="text-left text-hs-muted">
                <th className="p-2">Config</th>
                <th className="p-2">Scheduler</th>
                <th className="p-2">Makespan</th>
                <th className="p-2">TTFT p50</th>
                <th className="p-2">GPU util</th>
                <th className="p-2">Cost USD</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row, idx) => {
                const metrics = row.metrics as { makespan?: number; ttft_p50?: number; gpu_utilization?: number } | null;
                const benchmark = row.benchmark as { cost_usd?: number } | null;
                return (
                  <tr key={idx} className="border-t border-white/5">
                    <td className="p-2">{String(row.config)}</td>
                    <td className="p-2">{String(row.scheduler)}</td>
                    <td className="p-2">{metrics?.makespan?.toFixed(2) ?? "—"}</td>
                    <td className="p-2">{metrics?.ttft_p50?.toFixed(3) ?? "—"}</td>
                    <td className="p-2">{metrics ? `${(metrics.gpu_utilization! * 100).toFixed(1)}%` : "—"}</td>
                    <td className="p-2">{benchmark?.cost_usd?.toFixed(2) ?? "—"}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </Card>
    </div>
  );
}
