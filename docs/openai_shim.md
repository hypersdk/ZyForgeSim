# OpenAI-Compatible Virtual Endpoint (P6)

Planned feature: ForgeSim exposes a **virtual** OpenAI-compatible HTTP API for testing clients, workload generators, and AIPerf — **without running a real LLM**.

See the full roadmap: [benchmark_platform.md](benchmark_platform.md)

## Endpoint (planned)

```http
POST /v1/chat/completions
Authorization: Bearer <api-key>
Content-Type: application/json
```

## Internal flow

```text
HTTP request
  → parse model + messages → estimate input/output tokens
  → inject JobArrival into simulation queue
  → scheduler places on virtual GPUs
  → inference model (P1) computes TTFT + decode schedule
  → SSE stream of fake tokens (timing matches sim)
  → response complete
```

No GPU execution. Only scheduling and timing simulation.

## Security requirements (before ship)

These are **blockers** from architecture review — not optional polish:

| Requirement | Rationale |
|-------------|-----------|
| API key authentication | FastAPI `:8080` is reachable outside Next.js auth today |
| Rate limiting per key | Prevent DoS via unbounded sim fan-out |
| Bind `127.0.0.1` by default | Local dev only unless explicitly configured |
| No prompt logging in production | Privacy / secret leakage |

## AIPerf integration (P7)

AIPerf can target the shim as an OpenAI-compatible endpoint for **deterministic** benchmark runs in CI:

```text
AIPerf → ForgeSim OpenAI shim → simulated TTFT/TPS
```

Live AIPerf against real vLLM remains a separate **calibration** path (offline JSON import).

## Status

**Not implemented.** Target phase: **P6**, after P1 (inference model) and P2 (synthetic workloads).
