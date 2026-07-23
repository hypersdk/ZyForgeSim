"use client";

import { useCallback, useEffect, useState } from "react";
import Link from "next/link";
import { ComparePanel } from "@/components/ComparePanel";
import { Card, StatusBadge } from "@/components/ui";
import { compareConfigs, fetchConfigs, fetchRuns, startRun } from "@/lib/api";
import type { ConfigEntry, RunSummary, SimulationMetrics } from "@/types/simulation";

export default function HomePage() {
  const [configs, setConfigs] = useState<ConfigEntry[]>([]);
  const [runs, setRuns] = useState<RunSummary[]>([]);
  const [selected, setSelected] = useState("");
  const [compareA, setCompareA] = useState("");
  const [compareB, setCompareB] = useState("");
  const [compareResults, setCompareResults] = useState<
    Array<{ config: string; metrics: SimulationMetrics | null }>
  >([]);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    const [cfgs, runList] = await Promise.all([fetchConfigs(), fetchRuns()]);
    setConfigs(cfgs);
    setRuns(runList);
    if (!selected && cfgs.length) setSelected(cfgs[0].id);
  }, [selected]);

  useEffect(() => {
    refresh().catch(console.error);
    const t = setInterval(() => refresh().catch(console.error), 3000);
    return () => clearInterval(t);
  }, [refresh]);

  async function handleRun() {
    if (!selected) return;
    setBusy(true);
    try {
      const { id } = await startRun(selected);
      await refresh();
      window.location.href = `/runs/${id}`;
    } finally {
      setBusy(false);
    }
  }

  async function handleCompare() {
    if (!compareA || !compareB) return;
    setBusy(true);
    try {
      const { results } = await compareConfigs([compareA, compareB]);
      setCompareResults(results.map((r) => ({ config: r.config, metrics: r.metrics })));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="space-y-6">
      <Card title="Cluster Summary">
        <div className="flex flex-wrap items-end gap-3">
          <label className="text-sm text-slate-300">
            Config
            <select
              className="ml-2 rounded border border-slate-700 bg-slate-950 px-2 py-1"
              value={selected}
              onChange={(e) => setSelected(e.target.value)}
            >
              {configs.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.id}
                </option>
              ))}
            </select>
          </label>
          <button
            className="rounded bg-blue-600 px-4 py-2 text-sm font-medium disabled:opacity-50"
            disabled={busy || !selected}
            onClick={handleRun}
          >
            Run simulation
          </button>
        </div>
      </Card>

      <Card title="Recent Runs">
        <div className="overflow-x-auto">
          <table className="min-w-full text-sm">
            <thead className="text-xs uppercase text-slate-400">
              <tr>
                <th className="px-2 py-2 text-left">Config</th>
                <th className="px-2 py-2 text-left">Status</th>
                <th className="px-2 py-2 text-left">Created</th>
                <th className="px-2 py-2" />
              </tr>
            </thead>
            <tbody>
              {runs.map((run) => (
                <tr key={run.id} className="border-t border-slate-800">
                  <td className="px-2 py-2">{run.config}</td>
                  <td className="px-2 py-2">
                    <StatusBadge status={run.status} />
                  </td>
                  <td className="px-2 py-2 text-slate-400">{run.created_at.slice(0, 19)}</td>
                  <td className="px-2 py-2 text-right">
                    <Link className="text-blue-400 hover:underline" href={`/runs/${run.id}`}>
                      Open
                    </Link>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Card>

      <Card title="Compare Two Configs">
        <div className="mb-3 flex flex-wrap gap-3">
          <select className="rounded border border-slate-700 bg-slate-950 px-2 py-1" value={compareA} onChange={(e) => setCompareA(e.target.value)}>
            <option value="">Config A</option>
            {configs.map((c) => (
              <option key={c.id} value={c.id}>{c.id}</option>
            ))}
          </select>
          <select className="rounded border border-slate-700 bg-slate-950 px-2 py-1" value={compareB} onChange={(e) => setCompareB(e.target.value)}>
            <option value="">Config B</option>
            {configs.map((c) => (
              <option key={c.id} value={c.id}>{c.id}</option>
            ))}
          </select>
          <button className="rounded bg-slate-700 px-3 py-1 text-sm" disabled={busy} onClick={handleCompare}>
            Compare
          </button>
        </div>
        <ComparePanel results={compareResults} />
      </Card>
    </div>
  );
}
