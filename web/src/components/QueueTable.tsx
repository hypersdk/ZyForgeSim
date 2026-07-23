"use client";

import type { JobsTimeline } from "@/types/simulation";
import { Card } from "./ui";

export function QueueTable({ timeline }: { timeline: JobsTimeline | null }) {
  const jobs = timeline?.jobs ?? [];
  return (
    <Card title="Queue / Jobs">
      <div className="overflow-x-auto">
        <table className="min-w-full text-left text-sm">
          <thead className="text-xs uppercase tracking-wide text-hs-muted">
            <tr>
              <th className="px-2 py-2">Job</th>
              <th className="px-2 py-2">Priority</th>
              <th className="px-2 py-2">Tenant</th>
              <th className="px-2 py-2">GPUs</th>
              <th className="px-2 py-2">Status</th>
            </tr>
          </thead>
          <tbody>
            {jobs.map((job) => (
              <tr key={job.job_id} className="border-t border-hs-border">
                <td className="px-2 py-2 text-hs-heading">{job.name}</td>
                <td className="px-2 py-2 font-mono">{job.priority}</td>
                <td className="px-2 py-2 text-hs-muted">{job.tenant ?? "—"}</td>
                <td className="px-2 py-2 font-mono">{job.gpu_count}</td>
                <td className="px-2 py-2 capitalize text-hs-body">{job.state}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </Card>
  );
}
