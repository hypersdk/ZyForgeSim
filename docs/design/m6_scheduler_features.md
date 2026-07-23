# M6 — Forge scheduler features (scoping)

Status: quotas and priority done; gang plugin parity and preemption still
planned. This is a design scope, not a full implementation plan for the
remaining pieces — flags open questions to resolve before writing code.
Same format as [M5's scoping doc](m5_topology.md).

M6 bundles four mostly-independent features from `docs/milestones.md`:
quotas, priority, gang plugin parity, preemption. They don't need to land
together — recommend separate PRs in the order below.

## What exists today

- **Quotas — done.** `Cluster.tenant_quotas: HashMap<String, u32>` holds a
  per-tenant GPU cap; `Cluster::tenant_gpu_usage` sums GPUs held by a
  tenant's running jobs; `ResourceManager::can_place` rejects placement
  (holding the job in the waiting queue, not erroring) when placing it
  would push the tenant over quota. `FabricQuota.spec.gpuQuota.maxGPUs` is
  parsed into that map in `forge_bundle::load_forge_bundle`
  (`parse_tenant_quotas`); the internal YAML path accepts the same limits
  via `ClusterConfig.tenant_quotas`. See `docs/forge_input.md`'s "Tenant
  GPU quotas" section. Chose the "hold in queue" semantics from open
  question 1 below — `FifoScheduler` already skips jobs `can_place`
  rejects and keeps trying the rest of the queue, so no engine change was
  needed. Covered by unit tests in `resource.rs`/`cluster.rs` and
  `integration_forge_bundle_quota_delays_second_job`
  (`crates/forgesim-config/tests/integration.rs`), which proves two
  same-tenant jobs that *could* run concurrently on the raw GPU count
  instead serialize under a tight quota.
- **Priority — done.** `PriorityScheduler` (`crates/forgesim-scheduler/src/priority.rs`)
  sorts the waiting queue by `(priority desc, arrival_time asc)` via
  `Cluster::sort_waiting_by_priority`, then places jobs through the same
  `place_in_order` helper `FifoScheduler` now also shares
  (`crates/forgesim-scheduler/src/common.rs` — factored out since the two
  schedulers differed only in sort order). It does *not* preempt: a job
  already running when a higher-priority one arrives keeps running: the
  new job only wins the *next* scheduling decision, which is exactly the
  gap preemption (below) is meant to close. Selectable via
  `scheduler.type: priority` in internal YAML configs and
  `--scheduler priority` on `forge-sim run --forge-bundle` /
  `forge-sim replay`. Covered by unit tests in `priority.rs`/`cluster.rs`
  and `integration_priority_scheduler_prefers_high_priority_job`
  (`crates/forgesim-config/tests/integration.rs`), which runs the same
  workload under both policies and shows priority achieves a lower mean
  wait time for identical total makespan.
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

1. **What does "ForgeGang plugin parity" actually require?** The milestone
   name implies matching a specific Forge scheduler plugin's behavior, but
   nothing in this repo documents that plugin's placement policy (scoring,
   node grouping, min-available thresholds). Needs either a spec from the
   Forge side or a decision to scope this down to "atomic multi-node gang
   placement with node-count-aware bin packing," which is buildable without
   external input.
2. **Preemption trigger**: pure priority (any higher-priority waiting job
   can evict a lower-priority running one) vs. quota-driven (only evict to
   satisfy a tenant's own quota, never cross-tenant) vs. both. Needs a
   decision because it changes the fairness model entirely.
3. **Preemption mechanics**: does the engine need `EventQueue` support for
   removing/invalidating a pending `JobComplete` event, or is it simpler to
   let the stale `JobComplete` fire as a no-op (job already removed from
   `running_jobs`) and rely on `Cluster::finish_job`'s `Option` return
   already handling "job not found" gracefully? The latter avoids touching
   `EventQueue` at all — worth checking before assuming a queue API change
   is needed.
4. **Preempted job's re-arrival**: does it re-enter `waiting_queue` at its
   original `arrival_time` (preserves FIFO fairness, may starve if
   repeatedly preempted) or at preemption time (simpler, but a job could be
   preempted forever)? Needs a starvation-prevention answer either way
   (e.g. priority boost after N preemptions) or this will misbehave on
   contended clusters.
5. **Metrics**: `forgesim-metrics::SimulationMetrics` has no fields for
   preemption count or time spent preempted — needed to make it observable
   in `forge-sim run` output. (Quota holds are already observable
   indirectly via `mean_wait_time` / `makespan`, see the quota integration
   test.)

## Suggested order

1. ~~**Quotas**~~ — done, see above.
2. ~~**Priority scheduler**~~ — done, see above.
3. **Gang plugin parity** — blocked on open question 1. Until that's
   answered, no code to write here beyond what M2 already did.
4. **Preemption** — largest, touches the engine (`try_schedule` needs a
   path to evict running jobs, not just place waiting ones) and needs
   answers to questions 2–4 first. Do this last.

## Non-goals for M6

- Cross-cluster or hierarchical quotas (namespace + team + cluster-wide) —
  `FabricQuota` today is flat per-team; match that.
- Preemption cost modeling (e.g. checkpoint/restart overhead delaying the
  preempted job's eventual resume) — track as a possible M6 follow-up, not
  in the first slice.
