# M6 — Forge scheduler features (scoping)

Status: quotas, priority, preemption, and node-aware gang placement done.
Gang is scoped to atomic multi-node spread (`gang_size_nodes` distinct nodes)
rather than full ForgeGang plugin parity — see
[`resource.rs`](../../crates/forgesim-core/src/resource.rs).

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
- **Gang scheduling — done (scoped).** `gang_enabled` / `gang_size_nodes`
  are parsed from Forge annotations. `ResourceManager` requires gang jobs
  to spread `gpu_count / gang_size_nodes` GPUs across each of
  `gang_size_nodes` distinct nodes (all-or-nothing). This matches a
  buildable subset of Forge gang semantics without the full ForgeGang plugin
  spec. `ForgeScheduler` aliases the preemptive priority policy; quotas,
  gang spread, and topology are enforced by `ResourceManager`.
- **Gang timeout — done.** `forge.ai/gang-timeout` / `gang_timeout_secs`
  schedules a `GangTimeout` DES event at `arrival + timeout`. If the job is
  still waiting (could not be placed as a gang), it transitions to
  `JobState::Failed` and increments `SimulationMetrics.jobs_failed`. Covered
  by `engine::tests::gang_timeout_fails_job_that_never_fits` and
  `integration_gang_job_fails_when_gang_timeout_expires`.
- **Preemption — done.** `PreemptivePriorityScheduler`
  (`crates/forgesim-scheduler/src/preemptive.rs`) extends priority ordering:
  a waiting job that can't currently fit may evict lower-priority running
  jobs (lowest priority first) via the new `Cluster::evict_job` /
  `resume_evicted_job` pair, trying eviction candidates one at a time via
  `resource_manager.can_place` until either the job fits (commit — requeue
  the victims) or candidates run out (undo — restore everything evicted).
  Selectable via `scheduler.type: preemptive` / `--scheduler preemptive`,
  same as the other schedulers.

  Design decisions, resolving the open questions this doc used to list:
  - **Trigger** (was Q2): pure priority — a waiting job may only evict a
    *strictly lower priority* running one; quota is unrelated to
    preemption eligibility.
  - **Mechanics** (was Q3): resolved differently than originally
    suggested. The doc's original plan — let a stale `JobComplete` fire as
    a no-op once the job is no longer in `running_jobs` — turns out to be
    **unsafe**: if the same job is *resumed* before its stale event fires,
    that event finds the job back in `running_jobs` (just a different run)
    and would incorrectly finish it early, at the wrong time, freeing its
    GPU while the resumed run is still actually using it. Fixed with a
    `Job::run_generation` counter, bumped on every (re)start; `Event`
    carries the generation it completes, and
    `SimulationEngine::handle_complete` ignores an event whose generation
    doesn't match the job's current one. See
    `engine::tests::stale_job_complete_from_before_a_preemption_does_not_finish_the_resumed_run`
    for the regression test. `EventQueue` itself needed no removal API.
  - **Re-arrival / runtime accounting**: evicted jobs resume with
    *remaining* runtime, not a full restart — `Job::remaining_runtime`
    tracks seconds left, reduced by however long the last run segment
    lasted (`Job::requeue_after_preemption`). GPU utilization uses
    `Job::gpu_seconds_consumed` (segment sum across preemptions), not
    `(finish_time - start_time) * gpu_count`. Wait metrics use
    `cumulative_wait_secs` (queue-only time); legacy `wait_time()` still
    reports last-start minus arrival for backward compatibility.
  - **Gang timeout re-arm**: after preemption, gang jobs get a fresh
    `gang_deadline` and a new `GangTimeout` event; starting the job
    invalidates pending timeouts via `gang_timeout_generation`.
  - **Preemption decisions**: `job_preempted` entries in `decision_log`;
    tenant-scoped eviction when `quota_aware_preemption` is enabled (default).
  - **Restart penalty**: optional `SimulationEngine::preemption_restart_penalty_secs`
    delays resumed jobs (default 0).
  - **Starvation prevention** (was Q4): a job that's already been
    preempted `MAX_PREEMPTIONS` (3) times becomes exempt from further
    eviction, via `Job::preemption_count`.
  - **Scope**: only whole-GPU jobs trigger eviction (a MIG job that
    doesn't fit is left waiting, no eviction attempted) to avoid mixing
    preemption with MIG reconfiguration delay. Victims can still be MIG
    jobs.
  - **Metrics** (was Q5): `SimulationMetrics.preemptions` (from
    `Cluster.total_preemptions`) is now populated; the CLI prints a
    `preemptions:` line when nonzero, matching `mig_reconfigs`.

  Covered by scheduler-level unit tests (`preemptive.rs`: evicts for a
  higher-priority arrival, does not evict equal/higher priority, restores
  cleanly when eviction wouldn't free enough capacity), an engine-level
  regression test for the stale-event bug above, and
  `integration_preemptive_scheduler_evicts_for_higher_priority_arrival`
  (`crates/forgesim-config/tests/integration.rs`), which demonstrates the
  concrete case non-preemptive priority ordering can't handle: a
  low-priority job already *running* when a much higher-priority one
  arrives.

## Open questions

1. ~~**What does "ForgeGang plugin parity" actually require?**~~ Scoped down
   to node-aware all-or-nothing gang placement (implemented). Full ForgeGang
   plugin scoring/policy can be added later if a spec becomes available.

## Suggested order

1. ~~**Quotas**~~ — done, see above.
2. ~~**Priority scheduler**~~ — done, see above.
3. ~~**Preemption**~~ — done, see above.
4. ~~**Gang plugin parity**~~ — scoped and implemented as node-aware gang.
5. ~~**Gang timeout**~~ — implemented via `GangTimeout` events + `jobs_failed`.

## Non-goals for M6

- Cross-cluster or hierarchical quotas (namespace + team + cluster-wide) —
  `FabricQuota` today is flat per-team; match that.
- Preemption cost modeling (checkpoint/restart overhead delaying a
  preempted job's eventual resume) — the current model assumes free,
  instant checkpointing (resumed jobs pick up exactly where they left
  off). Modeling real restart overhead is a possible follow-up, not
  implemented.
- Quota-driven preemption (evicting to satisfy a tenant's own quota) —
  only priority-driven eviction is implemented.
