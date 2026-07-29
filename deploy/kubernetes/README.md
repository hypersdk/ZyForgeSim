# ForgeSim on Kubernetes

Deploy the ForgeSim web dashboard (FastAPI API + Next.js UI) to a Kubernetes cluster.

## Architecture

```text
Ingress (nginx)
  /api/auth/*  → forgesim-web:3000   (Next.js login)
  /api/*       → forgesim-api:8080   (FastAPI + Rust sim)
  /ws/*        → forgesim-api:8080   (WebSocket replay)
  /*           → forgesim-web:3000   (UI)
```

## Prerequisites

- Docker
- `kubectl` configured for your cluster
- NGINX Ingress Controller (`ingressClassName: nginx`)
- A default StorageClass for the API outputs PVC (optional but recommended)

## 1. Build images

From the repo root:

```bash
chmod +x deploy/build-images.sh
./deploy/build-images.sh
```

Push to your registry:

```bash
REGISTRY=registry.example.com/your-org TAG=0.1.0 PUSH=1 ./deploy/build-images.sh
```

Then edit [`kustomization.yaml`](kustomization.yaml) `images:` to match your registry/tag.

## 2. Configure auth

```bash
cd deploy/kubernetes
cp secret.example.yaml secret.yaml
# Edit FORGESIM_DASHBOARD_PASSWORD and FORGESIM_AUTH_SECRET
kubectl apply -f secret.yaml
```

## 3. Deploy

```bash
kubectl apply -k deploy/kubernetes
kubectl -n forgesim get pods
```

Edit [`ingress.yaml`](ingress.yaml) and set `host:` to your domain before applying, or patch after deploy.

## 4. Verify

Port-forward (no ingress):

```bash
kubectl -n forgesim port-forward svc/forgesim-web 3000:3000
kubectl -n forgesim port-forward svc/forgesim-api 8080:8080
```

Open http://localhost:3000 and log in with credentials from `secret.yaml`.

API health:

```bash
kubectl -n forgesim exec deploy/forgesim-api -- \
  python -c "import urllib.request; print(urllib.request.urlopen('http://127.0.0.1:8080/api/health').read())"
```

## Configuration

| Item | Location |
|------|----------|
| Cluster YAML configs | Baked into API image (`configs/`) |
| Run artifacts | PVC `forgesim-outputs` → `/app/outputs` |
| Dashboard login | Secret `forgesim-auth` |
| Ingress host | `ingress.yaml` |

To add custom cluster configs without rebuilding, mount a ConfigMap:

```yaml
volumeMounts:
  - name: extra-configs
    mountPath: /app/configs/clusters/custom
volumes:
  - name: extra-configs
    configMap:
      name: forgesim-extra-configs
```

## CLI-only batch job

Apply [`job.example.yaml`](job.example.yaml) for a one-off simulation, or:

```bash
kubectl apply -f deploy/kubernetes/job.example.yaml
kubectl -n forgesim logs job/forgesim-run-once
```

For interactive use, prefer the web UI or `POST /api/runs` on the API service.

## Notes

- **Run history** is in-memory in the API process; restarting the API pod clears the run list. Completed artifacts under `/app/outputs/runs` persist on the PVC.
- **Web rewrites** use `FORGESIM_API_URL=http://forgesim-api:8080` at image build time as a fallback when traffic goes through the Next.js pod instead of ingress path rules.
- **Production**: change default credentials; set a strong `FORGESIM_AUTH_SECRET`; use TLS on ingress.

## Teardown

```bash
kubectl delete -k deploy/kubernetes
kubectl delete -f deploy/kubernetes/secret.yaml
```
