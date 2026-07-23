"use client";

import { useEffect } from "react";
import type { ClusterSnapshot, SchedulerDecision } from "@/types/simulation";
import { useReplayStore } from "@/store/useReplayStore";
import { Card } from "./ui";

export function ReplayControls({
  snapshots,
  decisions,
}: {
  snapshots: ClusterSnapshot[];
  decisions: SchedulerDecision[];
}) {
  const { index, playing, speed, setSnapshots, setDecisions, setIndex, setPlaying, setSpeed, next, prev } =
    useReplayStore();

  useEffect(() => {
    setSnapshots(snapshots);
    setDecisions(decisions);
  }, [snapshots, decisions, setSnapshots, setDecisions]);

  useEffect(() => {
    if (!playing) return;
    const timer = setInterval(() => {
      const max = Math.max(snapshots.length, decisions.length, 1) - 1;
      if (index >= max) {
        setPlaying(false);
        return;
      }
      next();
    }, 500 / speed);
    return () => clearInterval(timer);
  }, [playing, speed, index, snapshots.length, decisions.length, next, setPlaying]);

  const decision = decisions[index] ?? decisions[decisions.length - 1];
  const snapshot = snapshots[index] ?? snapshots[snapshots.length - 1];

  return (
    <Card title="Scheduler Replay">
      <div className="mb-3 flex flex-wrap items-center gap-2">
        <button className="rounded bg-slate-700 px-3 py-1 text-sm" onClick={() => setPlaying(!playing)}>
          {playing ? "⏸ Pause" : "▶ Play"}
        </button>
        <button className="rounded bg-slate-700 px-3 py-1 text-sm" onClick={prev}>
          ⏮ Prev
        </button>
        <button className="rounded bg-slate-700 px-3 py-1 text-sm" onClick={next}>
          ⏭ Next
        </button>
        {[0.5, 1, 2, 10].map((s) => (
          <button
            key={s}
            className={`rounded px-2 py-1 text-xs ${speed === s ? "bg-blue-700" : "bg-slate-800"}`}
            onClick={() => setSpeed(s)}
          >
            {s}×
          </button>
        ))}
        <span className="text-xs text-slate-400">
          step {index + 1} / {Math.max(snapshots.length, decisions.length, 1)}
        </span>
      </div>
      {decision ? (
        <div className="rounded border border-slate-700 bg-slate-950/60 p-3 text-sm">
          <div className="text-slate-400">t={decision.time.toFixed(2)}s · {decision.kind}</div>
          <div className="mt-1 text-white">{decision.message}</div>
          {decision.gpu_ids.length > 0 && (
            <div className="mt-1 text-xs text-slate-400">GPUs: {decision.gpu_ids.join(", ")}</div>
          )}
        </div>
      ) : null}
      {snapshot ? (
        <div className="mt-2 text-xs text-slate-400">
          clock={snapshot.clock.toFixed(2)} · running={snapshot.running} · queued={snapshot.waiting}
        </div>
      ) : null}
    </Card>
  );
}
