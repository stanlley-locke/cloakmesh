# CloakMesh: Master Operations & Testing Manual (v5.0 - Final Milestone)

This is the definitive guide to exercising the entire CloakMesh ecosystem, from the core cryptographic engine to the custom Mullvad-based privacy browser.

---

## Part 1: Environment Synchronization

Ensure your system is ready for the high-performance polyglot build.

1.  **Open Terminal A: The Builder**
    ```bash
    # 1. Install all polyglot dependencies
    just bootstrap

    # 2. Generate synchronized Protobuf bindings
    just proto-gen

    # 3. Build the core Rust node in release mode
    cd core && cargo build --release
    ```

---

## Part 2: Hosting a Mesh Service (.cloak Site)

This procedure demonstrates **Programmatic Onion Service Provisioning** and **Decentralized Discovery**.

1.  **Terminal B: Start Local Site**
    ```bash
    # Create a real directory for your mesh site
    mkdir -p ~/mesh-site
    echo "<html><body><h1>CloakMesh Anonymous Node</h1><p>Metadata-resistant content.</p></body></html>" > ~/mesh-site/index.html

    # Start a local server (e.g., Python)
    python3 -m http.server 8080 --directory ~/mesh-site
    ```

2.  **Terminal A: Start your Gateway Node**
    ```bash
    ./target/release/cloakmesh --port 4001 --id main-gateway
    ```
    *   **CRITICAL:** Copy your `.cloak` address from the logs (e.g., `ahqw6...cloak`).

3.  **Terminal C: Configure Hosting**
    ```bash
    cd orchestrator
    # Map your .cloak identity to the local port 8080
    poetry run cloakcli node host <YOUR_ADDR> 8080

    # Publish your descriptor to the global DHT so others can find you
    poetry run cloakcli dht publish <YOUR_ADDR>
    ```

---

## Part 3: Anonymous Browsing & SOCKS5 Interception

Exercise **Three-Hop Circuit Architecture** and **Remote DNS Resolution**.

1.  **Terminal D: The Client (Browser/Curl)**
    ```bash
    # Visit your site through the 514-byte cell-framed onion tunnels
    curl -v -x socks5h://127.0.0.1:9050 http://<YOUR_ADDR>/index.html
    ```
    *   **Success Verification:** You should see the HTML content of your index.html file.

---

## Part 4: Secure Communication (Double Ratchet Chat)

Demonstrate **Asynchronous Mailboxes** and **Peer-to-Peer Communication**.

1.  **Terminal C: Start Chat Listener**
    ```bash
    poetry run cloakcli chat listen
    ```

2.  **Terminal D: Send Secure Message**
    ```bash
    # This message is encrypted via the Double Ratchet protocol
    poetry run cloakcli chat send "Encrypted update via onion mesh." --sender "RelayNode"
    ```

3.  **Terminal C: Retrieve History**
    ```bash
    poetry run cloakcli chat history
    ```

---

## Part 5: Encrypted File Sharing

Exercise **Fixed-Size Cell Framing** and **Metadata Stripping**.

1.  **Terminal C: Receive Mode**
    ```bash
    poetry run cloakcli file receive
    ```

2.  **Terminal D: Share File**
    ```bash
    echo "Confidential Mesh Data" > mesh_data.bin
    poetry run cloakcli file share mesh_data.bin <YOUR_ADDR>
    ```

3.  **Terminal C: Verify Received Files**
    ```bash
    poetry run cloakcli file list
    ```

---

## Part 6: Custom Privacy Browser (CloakBrowser)

Testing the CloakBrowser fork configuration.

1.  **Inspect Configuration:**
    *   Verify `cloak-browser/browser/app/profile/05-custom-network.js` for `network.proxy.socks_port 9050`.
2.  **Verify TLD Hook:**
    *   Inspect `cloak-browser/netwerk/dns/nsEffectiveTLDService.cpp` for `.cloak` and `.onion` overrides.
3.  **Build (Long Process):**
    ```bash
    cd cloak-browser
    # Ensure dependencies are installed (see TESTING_GUIDE_ADVANCED.md Step 1)
    ./mach build
    ./mach run
    ```

---

## Part 7: Verifying Advanced Privacy Features

1.  **Cell Framing (514B):** `cd core && cargo test traffic`
2.  **Reputation Scoring:** `poetry run cloakcli node circuits`
3.  **DHT Lookup:** `poetry run cloakcli dht fetch <ADDR>`

---

"Privacy isn't a feature. It's a foundation."
