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

The orchestrator now features a beautiful, fully-interactive Typer CLI menu!

### 1. Installation
Ensure you have `poetry` installed:
```bash
cd orchestrator
poetry install
```

### 2. Enter the Interactive Dashboard
Launch the unified interface that gives you access to all mesh commands:
```bash
poetry run cloakcli interactive
```
From here, you will see a rich, color-coded menu of all available features.

### 3. Build & Host Websites
```bash
# Inside the interactive menu
> site-init
# Generates a beautiful HTML project.

> host-static
# Starts a local web server, binds it to the mesh, and publishes to the DHT!
```

### 4. Browse the Mesh
```bash
> browse
# Enter any .cloak address to fetch it securely across the 3-hop proxy!
```

### 5. Chat & File Sharing
```bash
# Listen for incoming messages
> listen

# Send an encrypted message
> chat

# Share a file
> file-send
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
