# ForgeSim Web Dashboard

Next.js frontend for monitoring ForgeSim simulations.

**Full documentation:** [../docs/ui_dashboard.md](../docs/ui_dashboard.md)

## Quick start

From the repo root:

```bash
./scripts/setup_dev.sh
cd web && npm install && cd ..
./scripts/run_web_dashboard.sh
```

Open http://localhost:3000

## Scripts

| Script | Purpose |
|--------|---------|
| `./scripts/run_web_dashboard.sh` | API (8080) + UI (3000) together |
| `./scripts/run_web_api.sh` | FastAPI backend only |
| `./scripts/run_web_ui.sh` | Next.js frontend only |

Custom ports: `API_PORT=9000 UI_PORT=3001 ./scripts/run_web_dashboard.sh`

## Stack

- Next.js 14, React, TypeScript, Tailwind CSS
- Recharts (metrics), React Flow (topology), Zustand (replay state)
- Proxies `/api` and `/ws` to FastAPI on port 8080

## Development

See [docs/ui_dashboard.md](../docs/ui_dashboard.md) for API endpoints, run artifacts, troubleshooting, and production build notes.
