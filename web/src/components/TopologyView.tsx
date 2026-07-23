"use client";

import ReactFlow, { Background, Controls, Edge, Node } from "reactflow";
import "reactflow/dist/style.css";
import type { ClusterSnapshot } from "@/types/simulation";
import { Card } from "./ui";

export function TopologyView({ snapshot }: { snapshot: ClusterSnapshot | null }) {
  if (!snapshot?.nodes?.length) {
    return <Card title="GPU Topology">No topology data.</Card>;
  }

  const nodes: Node[] = [];
  const edges: Edge[] = [];
  let x = 0;
  snapshot.nodes.forEach((node, nodeIdx) => {
    node.gpus.forEach((gpu, gpuIdx) => {
      const id = gpu.id;
      nodes.push({
        id,
        position: { x: x + gpuIdx * 120, y: nodeIdx * 100 },
        data: { label: `${gpu.id}${gpu.busy ? `\n${gpu.job_name ?? "busy"}` : "\nidle"}` },
        style: {
          background: gpu.busy ? "#1e3a8a" : "#14532d",
          color: "#fff",
          border: "1px solid #475569",
          fontSize: 11,
          whiteSpace: "pre",
          width: 90,
        },
      });
      if (gpuIdx > 0) {
        edges.push({
          id: `${node.gpus[gpuIdx - 1].id}-${id}`,
          source: node.gpus[gpuIdx - 1].id,
          target: id,
          label: gpu.nvlink_group != null ? "NVLink" : "PCIe",
        });
      }
    });
    x += node.gpus.length * 120 + 40;
  });

  return (
    <Card title="GPU Topology">
      <div className="h-64 rounded border border-slate-800">
        <ReactFlow nodes={nodes} edges={edges} fitView proOptions={{ hideAttribution: true }}>
          <Background />
          <Controls />
        </ReactFlow>
      </div>
    </Card>
  );
}
