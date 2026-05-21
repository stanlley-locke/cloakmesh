# CloakMesh: Python Orchestrator & CLI Testing Suite

This document provides commands to verify the management, observability, and communication features of the Python `cloakcli`.

---

## 1. Automated Testing

Verify the CLI logic and Protobuf integration:
```bash
cd orchestrator
poetry run pytest
```

---

## 2. Feature-by-Feature CLI Verification

### Node Observability
```bash
# Check node health
poetry run cloakcli node status --port 4001

# View active onion circuits
poetry run cloakcli node circuits

# View peer reputation scores
poetry run cloakcli node reputation
```

### Decentralized Discovery (DHT)
```bash
# Publish a site descriptor
poetry run cloakcli dht publish <YOUR_ADDR>

# Fetch a descriptor
poetry run cloakcli dht fetch <YOUR_ADDR>
```

### Secure Identity (Capability Tokens)
```bash
# Issue a signed token for 'publish' scope
poetry run cloakcli auth issue <YOUR_ADDR> --scope "publish" --ttl 3600
```

### Bidirectional Communication (Double Ratchet)
```bash
# Start a real-time listener (Terminal B)
poetry run cloakcli chat listen

# Send a message (Terminal C)
poetry run cloakcli chat send "Hello through the mesh!" --sender "Stanlley"

# View message history
poetry run cloakcli chat history
```

### Encrypted File Sharing
```bash
# Enter receive mode
poetry run cloakcli file receive

# Share a local file
poetry run cloakcli file share sample.txt <TARGET_ADDR>

# List decrypted files
poetry run cloakcli file list
```

### Private Service Hosting
```bash
# Map .cloak address to local port 8080
poetry run cloakcli node host <ADDR> 8080
```

---

## 3. Protocol Parity Verification

Run the Python implementation of address derivation against the Rust results:
```bash
poetry run python tests/smoke_proto.py
```

"Privacy isn't a feature. It's a foundation."
