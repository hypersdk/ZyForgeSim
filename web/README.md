# ForgeSim Web Dashboard

Next.js frontend for monitoring ForgeSim simulations.

## Setup

```bash
cd web
npm install
```

## Development

Terminal 1 — FastAPI backend:

```bash
pip install -e '..[server]'
uvicorn forgesim.server.app:app --reload --port 8080
```

Terminal 2 — Next.js (proxies `/api` to port 8080):

```bash
cd web
npm run dev
```

Open http://localhost:3000
