# CloakMesh: Integrated Operational Manual (v8.0)

This manual provides the definitive, step-by-step procedure to initialize and verify the CloakMesh production ecosystem.

---

## Domain 1: Assembly (The Builder)

1.  **Environment Bootstrap:**
    ```bash
    just bootstrap
    just proto-gen
    ```

2.  **Binary Compilation:**
    ```bash
    # Build core engine
    cd core && cargo build --release

    # Build Admin UI
    cd cloak-admin && npm install && npm run build
    ```

---

## Domain 2: Orchestration (The Controller)

1.  **Terminal A: Start Node**
    ```bash
    cd core
    ./target/release/cloakmesh --port 4001 --id master-gateway
    ```
    *   **NOTE:** Record the `.cloak` address printed in the logs.

2.  **Terminal B: Network Registration**
    ```bash
    cd orchestrator
    poetry run cloakcli dht publish <YOUR_ADDR>
    ```

3.  **Terminal B: Site Hosting**
    ```bash
    # Host content on port 8080
    python3 -m http.server 8080 &
    poetry run cloakcli node host <YOUR_ADDR> 8080
    ```

---

## Domain 3: Verification (The Auditor)

1.  **Anonymous Routing (SOCKS5)**
    ```bash
    curl -v -x socks5h://127.0.0.1:9050 http://<YOUR_ADDR>/index.html
    ```

2.  **Secure P2P Communication**
    ```bash
    # Terminal B: Listen
    poetry run cloakcli chat listen

    # Terminal C: Send
    poetry run cloakcli chat send "Authenticated mesh update." "Stanlley"
    ```

---

## Domain 4: Observability (CloakAdmin)

1.  **Launch Dashboard:**
    ```bash
    cd cloak-admin
    npm run tauri dev
    ```

2.  **Audit Checklist:**
    *   **DASHBOARD:** Confirm White-Dominant, 0px-radius grid layout. Verify real-time bandwidth area charts and core health matrix.
    *   **IDENTITY:** Verify Ed25519 fingerprint display and key management action grid.
    *   **DISCOVERY:** Audit the real-time routing table and DHT objects.
    *   **CONSOLE:** Access the kernel-level log stream and CLI bridge.

---

"Privacy isn't a feature. It's a foundation."
