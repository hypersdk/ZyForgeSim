# Scheduler Benchmark Score (P4)

This document defines the ForgeSim scheduler benchmark report and optional composite score. Implementation is planned for **P4** of the [benchmark platform roadmap](benchmark_platform.md).

## Purpose

Compare scheduling policies on both **cluster efficiency** (utilization, fragmentation, fairness) and **serving quality** (TTFT, TPS, goodput) when inference jobs are present.

## Metric vector

Each run produces a `SchedulerBenchmarkReport` with the following fields:

| Category | Metric | Definition |
|----------|--------|------------|
| Scheduling | `makespan` | Simulation clock at last job completion |
| Scheduling | `mean_cumulative_wait` | Mean queue wait across completed jobs |
| Scheduling | `gpu_utilization` | GPU-seconds busy / (makespan × GPU count) |
| Scheduling | `queue_delay_p99` | 99th percentile queue delay (inference jobs) |
| Scheduling | `preemptions` | Total preemption count |
| Scheduling | `topology_penalties` | Cross-domain placement count |
| Serving | `ttft_p50`, `ttft_p99` | Time to first token percentiles (ms) — **not** `time_to_first_start` |
| Serving | `itl_p50` | Inter-token latency percentile (ms) |
| Serving | `tps_mean` | Mean decode tokens/sec |
| Serving | `goodput` | Fraction of requests meeting SLA (configurable TTFT/latency ceiling) |
| Fairness | `jain_index` | Jain fairness index across tenants |
| Efficiency | `fragmentation` | Idle GPU-time / total GPU-time |
| Cost | `gpu_hour_cost` | `gpu_seconds × rate` from `configs/analytics/cost.yaml` |

## Composite score (optional)

A single scalar is **optional** and must use **published weights** in config:

```yaml
# configs/analytics/score_weights.yaml (planned)
weights:
  ttft_p99: 0.25
  goodput: 0.25
  gpu_utilization: 0.20
  makespan: 0.15
  cost: 0.15
direction:
  ttft_p99: lower_is_better
  goodput: higher_is_better
  gpu_utilization: higher_is_better
  makespan: lower_is_better
  cost: lower_is_better
```

Default UI behavior (P4/P5): show the **full metric vector** and Pareto-style compare highlighting. Composite score is opt-in with tooltip showing weights.

## SLA / goodput

Goodput requires an SLA config per workload or tenant:

```yaml
sla:
  ttft_p99_ms: 500
  e2e_latency_p99_ms: 5000
```

A request counts toward goodput if simulated TTFT and end-to-end latency are both under the SLA at the concurrency observed when the job ran.

## Related docs

- [Benchmark platform roadmap](benchmark_platform.md)
- [M6 scheduler features](design/m6_scheduler_features.md)
