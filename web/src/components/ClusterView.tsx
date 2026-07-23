import type { ClusterSnapshot } from "@/types/simulation";
import { Card } from "./ui";

function gpuColor(gpu: ClusterSnapshot["nodes"][0]["gpus"][0]) {
  if (!gpu.busy) return "border-idle bg-green-950/40 text-green-200";
  if (gpu.utilization >= 0.95) return "border-overloaded bg-red-950/40 text-red-200";
  return "border-training bg-blue-950/40 text-blue-200";
}

export function ClusterView({ snapshot }: { snapshot: ClusterSnapshot | null }) {
  if (!snapshot) return <Card title="Cluster View">Waiting for snapshot…</Card>;

  return (
    <Card title="Cluster View">
      <div className="space-y-4">
        {snapshot.nodes.map((node) => (
          <div key={node.id}>
            <div className="mb-2 text-sm font-medium text-slate-300">{node.id}</div>
            <div className="grid grid-cols-2 gap-2 md:grid-cols-4">
              {node.gpus.map((gpu) => (
                <div
                  key={gpu.id}
                  className={`rounded border p-2 text-xs ${gpuColor(gpu)}`}
                  title={gpu.job_name ?? "idle"}
                >
                  <div className="font-semibold">{gpu.id}</div>
                  <div>{gpu.busy ? gpu.job_name ?? "busy" : "idle"}</div>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </Card>
  );
}
