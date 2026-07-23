"use client";

import ReactFlow, { Background, Controls, Edge, Node } from "reactflow";
import "reactflow/dist/style.css";
import type { ClusterSnapshot } from "@/types/simulation";
import { gpuStateColors, theme } from "@/lib/theme";
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
          background: gpu.busy ? gpuStateColors.busy : gpuStateColors.idle,
          color: theme.textBody,
          border: `1px solid ${theme.border}`,
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
          style: { stroke: theme.textMuted },
          labelStyle: { fill: theme.textMuted, fontSize: 10 },
        });
      }
    });
    x += node.gpus.length * 120 + 40;
  });

  return (
    <Card title="GPU Topology">
      <div className="h-64 rounded-hs border border-hs-border" style={{ backgroundColor: theme.surfaceCode }}>
        <ReactFlow nodes={nodes} edges={edges} fitView proOptions={{ hideAttribution: true }}>
          <Background color={theme.border} gap={16} />
          <Controls />
        </ReactFlow>
      </div>
    </Card>
  );
}
