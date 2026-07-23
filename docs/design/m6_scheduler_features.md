# M6 — Forge scheduler features (scoping)

Status: planned, not started. This is a design scope, not an implementation
plan — flags open questions to resolve before writing code. Same format as
[M5's scoping doc](m5_topology.md).

M6 bundles four mostly-independent features from `docs/milestones.md`:
quotas, priority, gang plugin parity, preemption. They don't need to land
together — recommend four separate PRs in the order below.

## What exists today

- **Quotas**: `FabricQuota` is parsed only to resolve a job's `tenant`
  string (`resolve_tenant`, `crates/forgesim-config/src/forge_bundle.rs:143`).
  `spec.gpuQuota.maxGPUs` (documented in `docs/forge_input.md:77`) is never
  read anywhere in the codebase — quotas are informational, not enforced.
  Nothing tracks per-tenant GPU usage.
- **Priority**: `Job.priority: u32` is parsed from `spec.priority` and
  stored, but `PriorityScheduler` is a stub that logs "not implemented" and
  returns no placements (`crates/forgesim-scheduler/src/stubs.rs`). Only
  `FifoScheduler` (arrival-time order) is real.
- **Gang scheduling**: `gang_enabled` / `gang_size_nodes` are parsed from
  Forge annotations and stored on `Job`, and gang jobs already get the
  correct total GPU count (`nodes × gpusPerNode`, M2). But nothing reads
  `gang_enabled`/`gang_size_nodes` after that — placement is "N total free
  GPUs anywhere in the cluster," not "N GPUs arranged across the requested
  node topology." Atomicity (all-or-nothing) is already implicit in
  `ResourceManager::allocate`, so basic gang semantics *may* already be
  sufficient depending on what "Forge gang plugin parity" needs to mean
  (see open questions). `ForgeScheduler` is an empty stub with a stale
  comment ("milestone 4").
- **Preemption**: no support at any layer. `JobState` has no `Preempted`
  variant (`Pending | Waiting | Running | Finished`,
  `crates/forgesim-core/src/models.rs`). More fundamentally,
  `SimulationEngine::try_schedule` (`crates/forgesim-core/src/engine.rs:84`)
  only ever calls `Scheduler::schedule` with the *waiting* queue — a
  `Scheduler` impl has no way to touch `running_jobs` or cause a running
  job to stop. And once a `JobComplete` event is pushed to the event queue
  at `start_job` time, it isn't cancelable — there's no event-removal API
  on `EventQueue`. This is an engine-level gap, not just a scheduler one.

## Open questions

1. **Quota enforcement point**: reject at ingest time (job never enters the
   waiting queue if it would exceed tenant quota — simple, but "rejected"
   isn't a `JobState` today either) vs. hold in queue until quota frees up
   (needs the scheduler to skip over quota-blocked jobs without deadlocking
   FIFO ordering for other tenants). Recommend the latter — it's the actual
   K8s admission behavior and is what a real Forge comparison run needs.
2. **What does "ForgeGang plugin parity" actually require?** The milestone
   name implies matching a specific Forge scheduler plugin's behavior, but
   nothing in this repo documents that plugin's placement policy (scoring,
   node grouping, min-available thresholds). Needs either a spec from the
   Forge side or a decision to scope this down to "atomic multi-node gang
   placement with node-count-aware bin packing," which is buildable without
   external input.
3. **Preemption trigger**: pure priority (any higher-priority waiting job
   can evict a lower-priority running one) vs. quota-driven (only evict to
   satisfy a tenant's own quota, never cross-tenant) vs. both. Needs a
   decision because it changes the fairness model entirely.
4. **Preemption mechanics**: does the engine need `EventQueue` support for
   removing/invalidating a pending `JobComplete` event, or is it simpler to
   let the stale `JobComplete` fire as a no-op (job already removed from
   `running_jobs`) and rely on `Cluster::finish_job`'s `Option` return
   already handling "job not found" gracefully? The latter avoids touching
   `EventQueue` at all — worth checking before assuming a queue API change
   is needed.
5. **Preempted job's re-arrival**: does it re-enter `waiting_queue` at its
   original `arrival_time` (preserves FIFO fairness, may starve if
   repeatedly preempted) or at preemption time (simpler, but a job could be
   preempted forever)? Needs a starvation-prevention answer either way
   (e.g. priority boost after N preemptions) or this will misbehave on
   contended clusters.
6. **Metrics**: `forgesim-metrics::SimulationMetrics` has no fields for
   quota rejections, preemption count, or time spent preempted — needed to
   make any of this observable in `forge-sim run` output.

## Suggested order

1. **Quotas** — smallest, most self-contained. Parse `gpuQuota.maxGPUs`
   into a per-tenant map on `Cluster` (or a new `QuotaTracker`), check it in
   `ResourceManager::can_place` alongside the existing GPU-availability
   check. No engine changes needed.
2. **Priority scheduler** — implement `PriorityScheduler` for real: sort
   `waiting_queue` by `(priority desc, arrival_time asc)` instead of
   `FifoScheduler`'s arrival-only sort. Almost a copy of `fifo.rs` with a
   different sort key — low risk.
3. **Gang plugin parity** — blocked on open question 2. Until that's
   answered, no code to write here beyond what M2 already did.
4. **Preemption** — largest, touches the engine (`try_schedule` needs a
   path to evict running jobs, not just place waiting ones) and needs
   answers to questions 3–5 first. Do this last.

## Non-goals for M6

- Cross-cluster or hierarchical quotas (namespace + team + cluster-wide) —
  `FabricQuota` today is flat per-team; match that.
- Preemption cost modeling (e.g. checkpoint/restart overhead delaying the
  preempted job's eventual resume) — track as a possible M6 follow-up, not
  in the first slice.
