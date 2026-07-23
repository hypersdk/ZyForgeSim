import type { JobsTimeline } from "@/types/simulation";
import { Card } from "./ui";

export function GanttChart({ timeline }: { timeline: JobsTimeline | null }) {
  if (!timeline?.jobs?.length) {
    return <Card title="Job Timeline">No jobs in timeline.</Card>;
  }

  const makespan = timeline.makespan || 1;
  const gpuIds = Array.from(
    new Set(timeline.jobs.flatMap((j) => j.assigned_gpus))
  ).sort();
  const rows = gpuIds.length ? gpuIds : [`gpu-0`];

  return (
    <Card title="Job Timeline (Gantt)">
      <div className="space-y-2">
        {rows.map((gpuId) => {
          const jobsOnGpu = timeline.jobs.filter((j) => j.assigned_gpus.includes(gpuId));
          return (
            <div key={gpuId} className="flex items-center gap-2 text-xs">
              <div className="w-16 shrink-0 text-slate-400">{gpuId}</div>
              <div className="relative h-6 flex-1 rounded bg-slate-950">
                {jobsOnGpu.map((job) => {
                  if (job.state === "failed") {
                    const left = (job.arrival_time / makespan) * 100;
                    const width = (((job.finish_time ?? job.arrival_time) - job.arrival_time) / makespan) * 100;
                    return (
                      <div
                        key={`${job.job_id}-failed`}
                        className="absolute top-1 h-4 rounded border border-dashed border-red-400 bg-red-900/50"
                        style={{ left: `${left}%`, width: `${Math.max(width, 1)}%` }}
                        title={`${job.name} (failed)`}
                      />
                    );
                  }
                  if (job.start_time == null) return null;
                  const waitLeft = (job.arrival_time / makespan) * 100;
                  const waitWidth = ((job.start_time - job.arrival_time) / makespan) * 100;
                  const runLeft = (job.start_time / makespan) * 100;
                  const runEnd = job.finish_time ?? job.start_time + job.runtime;
                  const runWidth = ((runEnd - job.start_time) / makespan) * 100;
                  return (
                    <span key={job.job_id}>
                      {waitWidth > 0 && (
                        <div
                          className="absolute top-1 h-4 rounded bg-orange-600/70"
                          style={{ left: `${waitLeft}%`, width: `${waitWidth}%` }}
                          title={`wait: ${job.name}`}
                        />
                      )}
                      <div
                        className="absolute top-1 h-4 rounded bg-teal-600/80"
                        style={{ left: `${runLeft}%`, width: `${Math.max(runWidth, 0.5)}%` }}
                        title={`run: ${job.name}`}
                      />
                    </span>
                  );
                })}
              </div>
            </div>
          );
        })}
      </div>
      <div className="mt-3 flex gap-4 text-xs text-slate-400">
        <span className="inline-flex items-center gap-1"><span className="h-2 w-4 rounded bg-orange-600/70" /> wait</span>
        <span className="inline-flex items-center gap-1"><span className="h-2 w-4 rounded bg-teal-600/80" /> run</span>
        <span className="inline-flex items-center gap-1"><span className="h-2 w-4 rounded border border-dashed border-red-400 bg-red-900/50" /> failed</span>
      </div>
    </Card>
  );
}
