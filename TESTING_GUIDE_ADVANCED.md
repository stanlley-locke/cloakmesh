# CloakMesh: Comprehensive Operations & Testing Manual

This manual provides a strict, step-by-step procedure to initialize, host, and interact with the CloakMesh network. Follow these steps in order to exercise the full system, including DHT management, capability-gated access, and anonymous browsing.

---

## Prerequisites & Environment Prep

Before starting, ensure all components are built and dependencies are in sync.

1.  **Terminal 1: Bootstrap Everything**
    ```bash
    # From the project root
    just bootstrap
    just proto-gen
    cd core && cargo build
    ```

---

## Step 1: Start the Core Network Node

The core node acts as your local gateway to the mesh. It handles identity, encryption, and the proxy bridge.

1.  **Terminal 1: Run Node**
    ```bash
    cd core
    # We use a specific node ID for consistency in this guide
    cargo run -- --port 4001 --id demo-gateway
    ```
2.  **Observe the Logs:**
    *   Find the `cloak_address` in the logs (e.g., `ahqw6...cloak`).
    *   Confirm `gRPC server starting, addr: 0.0.0.0:4001`.
    *   Confirm `SOCKS5 Proxy started ... addr: 127.0.0.1:9050`.

---

## Step 2: Secure Discovery (DHT Management)

In this step, we will register a "site" on the DHT so other nodes can find it.

1.  **Terminal 2: Publish Site Descriptor**
    ```bash
    cd orchestrator
    # Use the address found in Step 1. logs
    poetry run cloakcli dht publish <YOUR_CLOAK_ADDRESS>
    ```
    *   *Expected:* `SUCCESS: Descriptor for <ADDR> published successfully`.

2.  **Terminal 2: Fetch and Verify Site**
    ```bash
    poetry run cloakcli dht fetch <YOUR_CLOAK_ADDRESS>
    ```
    *   *Expected:* The CLI will display the public key and version found in the DHT.

---

## Step 3: Capability-Gated Access

Privileged actions require a signed token. We will issue one and verify it.

1.  **Terminal 2: Issue a 'publish' Token**
    ```bash
    poetry run cloakcli auth issue <YOUR_CLOAK_ADDRESS> --scope "publish" --ttl 3600
    ```
2.  **Verify the Token via Core Node:**
    *   (Note: The core currently verifies tokens internally during publication. To test the service explicitly):
    ```bash
    # Implement a small verification check via SDK (see Step 5)
    ```

---

## Step 4: Visiting .cloak Addresses (Mesh Proxy)

This is the ultimate test: using a standard tool (`curl`) to "visit" an anonymous address via the CloakMesh circuits.

1.  **Terminal 3: Browse the Mesh**
    ```bash
    # We tell curl to use our local proxy as a SOCKS5 gateway
    curl -v -x socks5h://127.0.0.1:9050 http://<YOUR_CLOAK_ADDRESS>/index.html
    ```
2.  **Observe the Interaction:**
    *   **Terminal 1 (Node):** You will see `New proxy connection` and `Tunneling proxy traffic through circuit`.
    *   **Terminal 3 (Curl):** You will receive a mock `HTTP 200 OK` response: *"Welcome to CloakMesh! This site is hosted on a .cloak address."*

---

## Step 5: Exercising Structured Error Codes

Force the system to fail to verify the robustness of the error reporting.

1.  **Test Case: Invalid Format (Code 2001)**
    ```bash
    cd orchestrator
    poetry run cloakcli dht fetch "not-an-address"
    ```
    *   *Output Detail:* `[2001] address: invalid format — missing .cloak suffix`.

2.  **Test Case: Address/Pubkey Mismatch (Code 5000)**
    ```bash
    # Try to publish for an address using a mismatched key (simulated in CLI)
    poetry run cloakcli dht publish some-other-node.cloak
    ```
    *   *Output Detail:* `status = StatusCode.UNAUTHENTICATED, details = "Address/Pubkey mismatch"`.

---

## Step 6: Automated Multi-Language Verification

Run the full validation suite to ensure parity between all implementations.

1.  **Terminal 2: Python Tests**
    ```bash
    cd orchestrator
    poetry run pytest
    poetry run python tests/smoke_proto.py
    ```

2.  **Terminal 3: TypeScript SDK Tests**
    ```bash
    cd sdk
    npx ts-node src/smoke_test.ts
    ```

---

## Step 7: Cleaning Up

1.  **Stop Node:** Press `Ctrl+C` in Terminal 1.
2.  **Verify Port Release:**
    ```bash
    netstat -tulpn | grep 4001
    ```

"Privacy isn't a feature. It's a foundation."
