#!/usr/bin/env bash
# Build and optionally push ForgeSim container images.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

REGISTRY="${REGISTRY:-}"
TAG="${TAG:-0.1.0}"
PUSH="${PUSH:-0}"
API_IMAGE="${API_IMAGE:-forgesim-api:${TAG}}"
WEB_IMAGE="${WEB_IMAGE:-forgesim-web:${TAG}}"
FORGESIM_API_URL="${FORGESIM_API_URL:-http://forgesim-api:8080}"

if [[ -n "$REGISTRY" ]]; then
  API_IMAGE="${REGISTRY}/forgesim-api:${TAG}"
  WEB_IMAGE="${REGISTRY}/forgesim-web:${TAG}"
fi

echo "Building API image: ${API_IMAGE}"
docker build -f deploy/docker/Dockerfile.api -t "$API_IMAGE" .

echo "Building Web image: ${WEB_IMAGE} (FORGESIM_API_URL=${FORGESIM_API_URL})"
docker build -f web/Dockerfile \
  --build-arg "FORGESIM_API_URL=${FORGESIM_API_URL}" \
  -t "$WEB_IMAGE" \
  web

if [[ "$PUSH" == "1" ]]; then
  echo "Pushing ${API_IMAGE}"
  docker push "$API_IMAGE"
  echo "Pushing ${WEB_IMAGE}"
  docker push "$WEB_IMAGE"
fi

cat <<EOF

Images ready:
  API: ${API_IMAGE}
  Web: ${WEB_IMAGE}

Deploy:
  cd deploy/kubernetes
  cp secret.example.yaml secret.yaml   # edit credentials
  kubectl apply -f secret.yaml
  kubectl apply -k .

Update image names in kustomization.yaml if using a private registry:
  images:
    - name: forgesim-api
      newName: ${API_IMAGE%:*}
      newTag: ${TAG}
EOF
