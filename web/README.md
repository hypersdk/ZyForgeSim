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

## Brand / theming

The web UI matches the [Zyvor](https://zyvor.dev) / HyperSDK dark palette:

| Token | Value | Usage |
|-------|-------|-------|
| `--hs-bg` | `#050505` | Page background |
| `--hs-accent` | `#f0583a` | Primary buttons, links |
| `--hs-indigo` | `#6366f1` | Active / busy GPU state |
| `--hs-teal` | `#10b981` | Gantt run segments |

- CSS variables: [`src/styles/zyvor-tokens.css`](src/styles/zyvor-tokens.css)
- Tailwind mapping: [`tailwind.config.ts`](tailwind.config.ts)
- Chart/topology constants: [`src/lib/theme.ts`](src/lib/theme.ts)
- Shared Python palette (CLI + matplotlib): [`../python/forgesim/theme.py`](../python/forgesim/theme.py)

Fonts: **Inter** (UI), **JetBrains Mono** (metrics). Header uses the Zyvor logo from `public/zyvor-logo.png`.

## Development

See [docs/ui_dashboard.md](../docs/ui_dashboard.md) for API endpoints, run artifacts, troubleshooting, and production build notes.
