# CloakMesh Deployment Guide

## Local Development (Docker Compose)

The fastest way to spin up a local CloakMesh testnet is using Docker Compose.

1.  **Build the images:**
    ```bash
    docker compose build
    ```

2.  **Start the network:**
    ```bash
    docker compose up -d
    ```
    This will start a 3-node testnet with a bootstrap node and two relays.

3.  **View logs:**
    ```bash
    docker compose logs -f
    ```

## Kubernetes Deployment

CloakMesh provides Helm charts for deployment to Kubernetes clusters.

### Prerequisites
- A running Kubernetes cluster.
- `helm` installed.

### Installation
1.  **Navigate to the Helm directory:**
    ```bash
    cd deploy/k8s/helm
    ```

2.  **Install the chart:**
    ```bash
    helm install cloakmesh . -f values.yaml
    ```

## Bare Metal / Systemd

For production-grade nodes on bare metal or VPS:

1.  **Build the release binary:**
    ```bash
    cd core && cargo build --release
    ```

2.  **Configure the node:**
    Copy `configs/default.toml` to `/etc/cloakmesh/config.toml` and edit as needed.

3.  **Setup Systemd service:**
    Use the provided template in `deploy/systemd/cloakmesh.service`.

## Monitoring

CloakMesh exports metrics in Prometheus format.
- **Port:** 9090 (default).
- **Endpoint:** `/metrics`.

Grafana dashboards are available in `deploy/monitoring/grafana/dashboards`.
