# M5 — Topology (scoping)

Status: **done** (domain grouping + runtime inflation). Domain-level NVLink
grouping with scatter fallback, `topology_penalties`, and
`topology_runtime_inflation` when jobs span domains. See
[`topology.rs`](../../crates/forgesim-core/src/topology.rs) and
[`resource.rs`](../../crates/forgesim-core/src/resource.rs).

## What exists today

Implemented in `forgesim-core`:

- `Gpu.nvlink_group: Option<u32>` — set by `ForgeBundleAdapter` as `i / 2`
  (pairs GPUs 0-1, 2-3, …) until Forge exports real topology.
- `HardwareProfile.nvlink_bw_gbs` / `pcie_bw_gbs` — feed `TopologyGraph` for
  runtime inflation when jobs span NVLink domains or nodes.
- `Job.network_bw_gbps` — jobs with this set (or `gang_enabled`) prefer
  same-`nvlink_group` placement in `ResourceManager::can_place_whole_gpu`.
- Scatter fallback increments `topology_penalties`; cross-domain placement
  inflates job duration via `Placement.runtime_multiplier`
  (`topology_runtime_inflation` in metrics).

## Goal

Model NVLink/PCIe (intra-node) and RDMA/network (inter-node) topology as a
graph derived from `FabricGpuNode`, and make placement locality-aware so
ForgeSim can answer: *does this scheduler colocate gang/distributed jobs on
well-connected GPUs, and does that matter for runtime?*

## Open questions (need answers before implementation)

1. **Where does real topology come from?** `FabricGpuNode.spec` today only
   has `nodeName`, `gpuCount`, `gpuType`, `memoryGB` (`docs/forge_input.md`).
   Does Forge export per-GPU NVLink adjacency, or just a topology *class*
   (e.g. "NVSwitch full-mesh" vs "PCIe-only")? If Forge doesn't expose this
   yet, M5 may need a synthetic topology generator (e.g. "8 GPUs/node,
   NVSwitch full mesh" as a hardware-profile-level assumption) rather than
   reading it from CRDs.
2. **Graph granularity**: per-GPU adjacency (accurate, more state) vs.
   per-node "NVLink domain" grouping (matches existing `nvlink_group`,
   cheaper). Recommend starting with domain-level grouping since that's what
   `nvlink_group` already models — extending to per-GPU adjacency is a
   later refinement if domain-level proves insufficient.
3. **Scoring vs. hard constraint**: should the scheduler *require* gang jobs
   to fit within one NVLink domain (reject/wait otherwise), or just prefer
   it and fall back to cross-domain placement with a runtime penalty? The
   latter needs a cost model (e.g. inflate `runtime` when GPUs span domains
   at less than `network_bw_gbps`); the former is simpler but may starve
   jobs on fragmented clusters.
4. **Where does this live?** Likely a new `Topology` type in
   `forgesim-core` (graph over `Node`/`Gpu`) plus a `ResourceManager` change
   to make placement topology-aware, plus a scheduler-visible cost/penalty —
   touches `resource.rs`, `cluster.rs`, `forge_bundle.rs`, and whichever
   scheduler(s) opt into locality-aware placement.
5. **Metrics**: `forgesim-metrics` would need a new signal (e.g. "% of
   distributed jobs placed within a single NVLink domain") to make M5
   observable in `forge-sim run` output.

## Suggested first slice

Domain-level grouping (question 2) + hard-constraint-with-fallback
(question 3): extend `ResourceManager::can_place_whole_gpu` to prefer
same-`nvlink_group` placement for jobs with `gang_enabled` or
`network_bw_gbps` set, falling back to today's scatter behavior with a
`topology_penalty` counter in metrics. Keeps the change scoped to
`resource.rs` + `metrics` without inventing a full graph type yet, and
gives a concrete signal to decide if per-GPU adjacency (vs. domain-level)
is actually needed.

## Non-goals for M5

- Real NVLink/PCIe topology *export* from a live Forge cluster (depends on
  what Forge's CRDs actually expose — a prerequisite question, not part of
  ForgeSim itself).
- Multi-rack / multi-datacenter network modeling — single-cluster only, per
  the existing `Cluster` model.
