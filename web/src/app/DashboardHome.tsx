"use client";

import { useCallback, useEffect, useState } from "react";
import { ComparePanel } from "@/components/ComparePanel";
import { AppLink, Button, Card, EmptyState, FormField, PageHero, Select, StatusBadge } from "@/components/ui";
import { compareConfigs, fetchConfigs, fetchRuns, startRun } from "@/lib/api";
import type { ConfigEntry, RunSummary, SimulationMetrics } from "@/types/simulation";

export function DashboardHome() {
  const [configs, setConfigs] = useState<ConfigEntry[]>([]);
  const [runs, setRuns] = useState<RunSummary[]>([]);
  const [selected, setSelected] = useState("");
  const [compareA, setCompareA] = useState("");
  const [compareB, setCompareB] = useState("");
  const [compareResults, setCompareResults] = useState<
    Array<{ config: string; metrics: SimulationMetrics | null }>
  >([]);
  const [busy, setBusy] = useState(false);

  const sameCompareConfig = Boolean(compareA && compareB && compareA === compareB);
  const canCompare = Boolean(compareA && compareB && compareA !== compareB);

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

  useEffect(() => {
    if (sameCompareConfig) setCompareResults([]);
  }, [sameCompareConfig]);

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
    if (!compareA || !compareB || compareA === compareB) return;
    setBusy(true);
    try {
      const { results } = await compareConfigs([compareA, compareB]);
      setCompareResults(results.map((r) => ({ config: r.config, metrics: r.metrics })));
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <PageHero
        kicker="GPU Scheduler Simulator"
        title="Dashboard"
        subtitle="Launch simulations, inspect cluster state, replay scheduler decisions, and compare scheduling policies side by side."
      />

      <div className="dashboard-grid">
        <Card
          variant="action"
          title="Cluster Summary"
          description="Pick a cluster config and start a new simulation run."
        >
          <div className="form-row">
            <FormField label="Configuration">
              <Select value={selected} onChange={(e) => setSelected(e.target.value)}>
                {configs.map((c) => (
                  <option key={c.id} value={c.id}>
                    {c.id}
                  </option>
                ))}
              </Select>
            </FormField>
            <Button disabled={busy || !selected} onClick={handleRun}>
              {busy ? "Starting…" : "Run simulation"}
            </Button>
          </div>
        </Card>

        <Card title="Recent Runs" description="Latest simulation jobs from this session.">
          {runs.length === 0 ? (
            <EmptyState
              title="No runs yet"
              text="Start a simulation above to populate this table with live status and metrics."
            />
          ) : (
            <div className="data-table-wrap">
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Config</th>
                    <th>Status</th>
                    <th>Created</th>
                    <th />
                  </tr>
                </thead>
                <tbody>
                  {runs.map((run) => (
                    <tr key={run.id}>
                      <td className="font-medium text-hs-heading">{run.config}</td>
                      <td>
                        <StatusBadge status={run.status} />
                      </td>
                      <td className="font-mono text-xs text-hs-muted">{run.created_at.slice(0, 19)}</td>
                      <td className="text-right">
                        <AppLink href={`/runs/${run.id}`}>Open</AppLink>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </Card>

        <Card title="Compare Two Configs" description="Run both configs and compare scheduling metrics side by side.">
          <div className="form-row">
            <FormField label="Config A">
              <Select value={compareA} onChange={(e) => setCompareA(e.target.value)}>
                <option value="">Select config…</option>
                {configs.map((c) => (
                  <option key={c.id} value={c.id} disabled={c.id === compareB}>
                    {c.id}
                  </option>
                ))}
              </Select>
            </FormField>
            <FormField label="Config B">
              <Select value={compareB} onChange={(e) => setCompareB(e.target.value)}>
                <option value="">Select config…</option>
                {configs.map((c) => (
                  <option key={c.id} value={c.id} disabled={c.id === compareA}>
                    {c.id}
                  </option>
                ))}
              </Select>
            </FormField>
            <Button variant="secondary" disabled={busy || !canCompare} onClick={handleCompare}>
              Compare
            </Button>
          </div>
          {sameCompareConfig ? (
            <p className="compare-hint">Choose two different configs — comparing the same file won&apos;t show any difference.</p>
          ) : null}
          <ComparePanel results={compareResults} />
        </Card>
      </div>
    </>
  );
}
