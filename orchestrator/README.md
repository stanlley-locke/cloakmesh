# CloakMesh: Python Orchestrator CLI

The production-grade management and communication interface for CloakMesh nodes.

## 🛠️ Features

*   **Node Management**: Lifecycle control (`start`, `stop`, `status`).
*   **Observability**: Real-time monitoring of **Onion Circuits** and **Peer Reputation**.
*   **Decentralized Discovery**: Direct interaction with the DHT (`publish`, `fetch`).
*   **Mesh Communication**:
    *   **Chat**: Bidirectional streaming chat with **Double Ratchet** encryption.
    *   **File Transfer**: Chunked streaming for secure file sharing.
*   **Identity**: Issue signed **Capability Tokens** for gated access.

## 🚀 Usage

### 1. Installation
Ensure you have `poetry` installed:
```bash
cd orchestrator
poetry install
```

### 2. Monitoring the Mesh
```bash
# View active onion tunnels
poetry run cloakcli node circuits

# View relay trust scores
poetry run cloakcli node reputation
```

### 3. P2P Communication
```bash
# Start listening for messages
poetry run cloakcli chat listen

# Send a message
poetry run cloakcli chat send "Your message" "SenderName"
```

### 4. Hosting & Discovery
```bash
# Map local server (port 80) to .cloak identity
poetry run cloakcli node host your-address.cloak 80

# Announce yourself to the DHT
poetry run cloakcli dht publish your-address.cloak
```

## 🧪 Testing
```bash
# Run unit and integration tests
poetry run pytest

# Run protocol smoke test
poetry run python tests/smoke_proto.py
```

---
"Privacy isn't a feature. It's a foundation."
