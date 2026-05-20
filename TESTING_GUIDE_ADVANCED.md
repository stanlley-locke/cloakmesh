# CloakMesh Milestone: Structured Errors & Production Use Cases

This guide demonstrates how to exercise the production-ready features of CloakMesh, including structured error handling and decentralized use cases.

## 1. Structured Error Codes

The system now returns unified error codes (1000-6000 range).

### Observe a Protocol Error (Invalid Address)
Try to fetch a descriptor for an invalid address using the Python CLI.
```bash
cd orchestrator
poetry run cloakcli dht fetch invalid-addr
```
*Expected Output:* An error message containing `[INVALID_ADDRESS (2001)]` or similar structured format.

### Observe an Auth Error (Missing Token)
Try to publish a descriptor without correct authorization (requires implementation of token passing in CLI).
```bash
# This will trigger a Status::unauthenticated in the core
poetry run cloakcli dht publish test.cloak
```

---

## 2. Production Use Cases

### Use Case 1: Secure Descriptor Management (DHT)
Services can publish their location, and clients can discover them via the Kademlia DHT.

1.  **Publish a Descriptor:**
    ```bash
    # Address must match derived address of node (or mocked for demo)
    poetry run cloakcli dht publish ahqw6zrrljnem7gxqlducifffw2v7nhgyqujcy36jlfwr5xbmaxfg3iuxdoa.cloak
    ```
2.  **Fetch the Descriptor:**
    ```bash
    poetry run cloakcli dht fetch ahqw6zrrljnem7gxqlducifffw2v7nhgyqujcy36jlfwr5xbmaxfg3iuxdoa.cloak
    ```

### Use Case 2: Capability-Gated Access
Privileged operations are protected by signed capability tokens.

1.  **Issue a Token (Local):**
    ```bash
    poetry run cloakcli auth issue ahqw6zrrljnem7gxqlducifffw2v7nhgyqujcy36jlfwr5xbmaxfg3iuxdoa.cloak --scope "publish"
    ```
2.  **Verify via Core Node:**
    The core node now internally verifies these tokens during `PublishDescriptor` operations.

---

## 3. Developer SDK Utilization

The TypeScript SDK now supports these high-level operations.

### Fetch Descriptor via SDK
```typescript
import { CloakClient } from './src/client/grpc_client';

async function run() {
    const client = new CloakClient();
    try {
        const descriptor = await client.fetchDescriptor('ahqw6zrrljnem7gxqlducifffw2v7nhgyqujcy36jlfwr5xbmaxfg3iuxdoa.cloak');
        console.log('Descriptor Found:', descriptor);
    } catch (error) {
        console.error('DHT Lookup Failed:', error);
    }
}
```

---
"Privacy isn't a feature. It's a foundation."
