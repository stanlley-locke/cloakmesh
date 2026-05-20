# CloakMesh Threat Model

## Assumptions

1.  **Adversary Capabilities:** The adversary can observe all network traffic, compromise a fraction of the relays, and inject malicious nodes (Sybil attack).
2.  **Cryptographic Security:** Standard primitives (Ed25519, X25519, AES/ChaCha) are considered secure unless broken by a quantum computer (mitigated by hybrid PQ crypto).
3.  **Trusted Local Environment:** The user's local machine and the CloakMesh Core process are trusted.

## Mitigations

### 1. Traffic Analysis
- **Threat:** An ISP or global observer correlates traffic patterns to deanonymize users.
- **Mitigation:** Fixed-size cell padding, jitter injection, and cover flow traffic.

### 2. Path Bias / Relay Compromise
- **Threat:** An adversary compromises a series of relays in a circuit.
- **Mitigation:** Adaptive 3-6 hop circuits, reputation-weighted relay selection, and frequent circuit rotation.

### 3. Sybil Attacks
- **Threat:** An adversary spawns thousands of malicious nodes to dominate the DHT or relay pool.
- **Mitigation:** Stake-backed trust scores, proof-of-work puzzles for node registration, and social graph validation.

### 4. Descriptor Poisoning
- **Threat:** Malicious DHT nodes return fake or stale descriptors.
- **Mitigation:** Merkle-verifiable storage proofs and mandatory Ed25519 signatures on all descriptors.

### 5. Post-Quantum Vulnerability
- **Threat:** A quantum computer breaks X25519 key exchange.
- **Mitigation:** Hybrid key exchange combining X25519 with Kyber-768 (Phase 2+).

## Residual Risks

- **Endpoint Compromise:** If the client or service machine is compromised, the protocol cannot protect the data.
- **Application-Layer Leaks:** Users may leak identity via behavioral patterns or unencrypted application data.
- **Low-Latency Correlation:** While mitigated, high-resource global observers may still perform statistical correlation over long periods.
