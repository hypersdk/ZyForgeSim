import type { ClusterSnapshot } from "@/types/simulation";
import { Card } from "./ui";

function gpuColor(gpu: ClusterSnapshot["nodes"][0]["gpus"][0]) {
  if (!gpu.busy) return "border-idle bg-hs-success/10 text-hs-success-light border-hs-success/30";
  if (gpu.utilization >= 0.95) return "border-overloaded bg-hs-error/10 text-hs-error-light border-hs-error/30";
  return "border-training bg-hs-indigo/10 text-hs-purple-light border-hs-indigo/30";
}

export function ClusterView({ snapshot }: { snapshot: ClusterSnapshot | null }) {
  if (!snapshot) return <Card title="Cluster View">Waiting for snapshot…</Card>;

  return (
    <Card title="Cluster View">
      <div className="space-y-4">
        {snapshot.nodes.map((node) => (
          <div key={node.id}>
            <div className="mb-2 text-sm font-medium text-hs-heading">{node.id}</div>
            <div className="grid grid-cols-2 gap-2 md:grid-cols-4">
              {node.gpus.map((gpu) => (
                <div
                  key={gpu.id}
                  className={`rounded-hs border p-2 font-mono text-xs ${gpuColor(gpu)}`}
                  title={gpu.job_name ?? "idle"}
                >
                  <div className="font-semibold">{gpu.id}</div>
                  <div>{gpu.busy ? (gpu.job_name ?? "busy") : "idle"}</div>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </Card>
  );
}
