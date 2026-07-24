"use client";

import type { JobsTimeline } from "@/types/simulation";
import { Card } from "./ui";

export function QueueTable({ timeline }: { timeline: JobsTimeline | null }) {
  const jobs = timeline?.jobs ?? [];
  return (
    <Card
      title="Queue / Jobs (final summary)"
      description="Final job states for the completed run. Does not update during scheduler replay."
    >
      <div className="data-table-wrap">
        <table className="data-table">
          <thead>
            <tr>
              <th>Job</th>
              <th>Priority</th>
              <th>Tenant</th>
              <th>GPUs</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            {jobs.length === 0 ? (
              <tr>
                <td colSpan={5} className="text-center text-hs-muted">
                  No jobs recorded.
                </td>
              </tr>
            ) : (
              jobs.map((job) => (
                <tr key={job.job_id}>
                  <td className="font-medium text-hs-heading">{job.name}</td>
                  <td className="font-mono">{job.priority}</td>
                  <td className="text-hs-muted">{job.tenant ?? "—"}</td>
                  <td className="font-mono">{job.gpu_count}</td>
                  <td className="capitalize text-hs-body">{job.state}</td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </Card>
  );
}
