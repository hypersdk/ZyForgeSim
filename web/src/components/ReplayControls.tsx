"use client";

import { useEffect } from "react";
import type { ClusterSnapshot, SchedulerDecision } from "@/types/simulation";
import { useReplayStore } from "@/store/useReplayStore";
import { Button, Card } from "./ui";
import clsx from "clsx";

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
        <Button variant="secondary" onClick={() => setPlaying(!playing)}>
          {playing ? "⏸ Pause" : "▶ Play"}
        </Button>
        <Button variant="secondary" onClick={prev}>
          ⏮ Prev
        </Button>
        <Button variant="secondary" onClick={next}>
          ⏭ Next
        </Button>
        {[0.5, 1, 2, 10].map((s) => (
          <button
            key={s}
            className={clsx(
              "rounded-hs px-2 py-1 text-xs transition",
              speed === s
                ? "bg-hs-indigo/30 text-hs-purple-light border border-hs-indigo/40"
                : "bg-hs-bg border border-hs-border text-hs-muted hover:border-hs-border-accent"
            )}
            onClick={() => setSpeed(s)}
          >
            {s}×
          </button>
        ))}
        <span className="font-mono text-xs text-hs-muted">
          step {index + 1} / {Math.max(snapshots.length, decisions.length, 1)}
        </span>
      </div>
      {decision ? (
        <div className="rounded-hs border border-hs-border bg-hs-surface-code/60 p-3 text-sm">
          <div className="font-mono text-xs text-hs-muted">
            t={decision.time.toFixed(2)}s · {decision.kind}
          </div>
          <div className="mt-1 text-hs-heading">{decision.message}</div>
          {decision.gpu_ids.length > 0 && (
            <div className="mt-1 font-mono text-xs text-hs-muted">GPUs: {decision.gpu_ids.join(", ")}</div>
          )}
        </div>
      ) : null}
      {snapshot ? (
        <div className="mt-2 font-mono text-xs text-hs-muted">
          clock={snapshot.clock.toFixed(2)} · running={snapshot.running} · queued={snapshot.waiting}
        </div>
      ) : null}
    </Card>
  );
}
