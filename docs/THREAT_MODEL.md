# CloakMesh Threat Model

This document defines the adversaries CloakMesh is designed to resist, the attacks it considers in-scope, and the explicit limitations of the current implementation.

---

## Threat Model Goals

CloakMesh protects two primary properties:

1. **Sender Anonymity** — An observer watching the network cannot determine who sent a message, requested a resource, or initiated a connection.
2. **Receiver Anonymity** — An observer cannot determine which hidden service is being contacted or where it is hosted.

Both properties must hold simultaneously (relationship unlink-ability) even when the adversary controls significant fractions of the network.

---

## Adversary Model

### Adversary A1: Global Passive Adversary (GPA)
Can observe all network traffic (timing, packet sizes, volumes) but cannot actively modify it.

**Example:** Nation-state-level DPI, backbone tap.

| CloakMesh defense | Status |
|---|---|
| Fixed-size cells (256/1024B) | ✅ Implemented |
| Cover traffic (dummy cells at idle) | ✅ Implemented |
| Timing jitter on cell emission | ✅ Implemented |
| Multi-hop routing (3 hops minimum) | ✅ Implemented |
| Guard node pinning (long-term stability) | ✅ Implemented |

**Residual risk:** A sufficiently powerful GPA can still perform long-range timing correlation attacks across all hops. Defense requires more hops or more cover traffic — roadmapped.

---

### Adversary A2: Local Network Adversary
Controls or observes the network link between a target and their guard node.

**Example:** ISP, university firewall, coffee shop WiFi.

| CloakMesh defense | Status |
|---|---|
| All traffic encrypted (ChaCha20-Poly1305) | ✅ |
| All traffic looks like gRPC/HTTP2 (no protocol fingerprint) | ✅ |
| Fixed cell size hides payload length | ✅ |
| Timing jitter reduces correlation | ✅ |

**Note:** The local adversary can tell you're using CloakMesh (gRPC traffic to a known port). Traffic obfuscation (bridge nodes, domain fronting) is a Phase 6 feature.

---

### Adversary A3: Malicious Relay Node
A node controlled by the adversary that is inserted into onion circuits.

**Example:** Adversary runs 100 nodes hoping to be selected as Guard + Exit simultaneously.

| CloakMesh defense | Status |
|---|---|
| 3-hop minimum (adversary needs Guard AND Exit) | ✅ |
| Guard node pinning (resistant to churning attacks) | ✅ |
| No plaintext visible to relay (each hop sees only next/prev) | ✅ |
| Integrity tags (ChaCha20-Poly1305) detect tampering | ✅ |
| Reputation system (low-rep nodes deprioritized) | ⚠️ Partial |

**Residual risk:** An adversary controlling both the Guard and Exit/Rendezvous nodes can correlate timing to de-anonymize a connection. This is the classic Sybil circuit-level attack. Mitigated by guard node pinning and reputation scoring.

---

### Adversary A4: Malicious DHT Peer (Sybil Attack)
Adversary injects many nodes to gain control of a DHT key range and prevent descriptor lookups or return false descriptors.

| CloakMesh defense | Status |
|---|---|
| Cryptographically verified descriptors (signed by service) | ✅ |
| Multiple intro points per descriptor | ✅ |
| Redundant DHT replication (K=20 per bucket) | ✅ |
| Reputation-weighted peer selection | ⚠️ Partial |
| Proof-of-work for DHT write access | 📅 Roadmap |

**Residual risk:** Without Proof-of-Work, Sybil attacks can eclipse small DHT regions. Full Sybil resistance requires PoW or stake-based admission (Phase 5).

---

### Adversary A5: Compromised Hidden Service
The `.cloak` service itself is compromised or coerced.

| CloakMesh defense | Status |
|---|---|
| Volatile RAM storage (session keys never on disk) | ✅ |
| Zeroize on drop (keys erased from memory on session end) | ✅ |
| No logs of client identities or IPs | ✅ |
| Introduction points don't know clients | ✅ |

**Residual risk:** A compromised service can reveal its own DHT descriptor and intro points. It cannot reveal client identities (which were never visible to it).

---

### Adversary A6: Passive Traffic Analysis (Machine Learning)
Uses ML to classify traffic patterns, detect CloakMesh usage, or de-anonymize flows despite encryption.

| CloakMesh defense | Status |
|---|---|
| Fixed cell size (removes packet-length features) | ✅ |
| Cover traffic (removes silence as signal) | ✅ |
| Timing jitter (reduces inter-arrival time features) | ✅ |
| Traffic obfuscation (pluggable transports) | 📅 Phase 6 |

---

## Out-of-Scope Threats

These threats are **explicitly not** in CloakMesh's scope:

| Threat | Reason |
|---|---|
| **Endpoint security** | If your device is compromised, no anonymity system can help |
| **Browser fingerprinting** | Requires CloakBrowser (separate project) |
| **Social engineering** | Protocol cannot protect against human error |
| **Physical surveillance** | Out of scope for software |
| **Legal compulsion of relay operators** | Operators can't reveal what they don't store |
| **Quantum adversaries (current)** | Kyber KEM in handshakes is a defense; broader PQ posture is ongoing |

---

## Privacy Properties Table

| Property | Current Status | Phase Target |
|---|---|---|
| Sender anonymity | ✅ 3-hop onion routing | Phase 2 |
| Receiver anonymity | ✅ Descriptors + intro points | Phase 2 |
| Forward secrecy | ✅ Ephemeral X25519 per session | Phase 2 |
| Per-message forward secrecy | 📅 Double Ratchet | Phase 4 |
| Post-quantum secrecy | ✅ Kyber-768 hybrid | Phase 2 |
| Traffic analysis resistance | ✅ Fixed cells, cover traffic | Phase 2 |
| Metadata resistance | ✅ No PII logged | Phase 1 |
| Relationship unlink-ability | ⚠️ Partial (timing correlation) | Phase 5 |
| Censorship resistance | ✅ DHT, no central directory | Phase 2 |
| Plausible deniability (relay) | ✅ Relay node sees only encrypted cells | Phase 2 |

---

## Security Assumptions

CloakMesh's security rests on the following computational hardness assumptions:

1. **Ed25519:** Security relies on the discrete logarithm problem over the Edwards curve (Curve25519). Classical security: 128 bits.
2. **X25519:** Security relies on Computational Diffie-Hellman on Curve25519. Classical security: 128 bits.
3. **Kyber-768:** Security relies on Module Learning With Errors (MLWE). Post-quantum security: ≥ 178 bits.
4. **ChaCha20-Poly1305:** Security relies on the ChaCha20 stream cipher and Poly1305 MAC. Classical security: 128 bits (authentication).
5. **SHA-256:** Second preimage resistance and collision resistance. Security: 128 bits (collision), 256 bits (preimage).

All crates are pinned and reviewed for unsafe code. See [CRYPTO.md](./CRYPTO.md#cryptographic-dependency-inventory) for the full dependency list.

---

## Responsible Disclosure

If you discover a security vulnerability in CloakMesh, please report it via the process described in [SECURITY.md](../SECURITY.md). Do not open public issues for security vulnerabilities.

**Critical severity:** Cryptographic breaks, anonymity de-anonymization attacks, remote code execution.  
**High severity:** DHT poisoning, descriptor forgery, capability token bypass.  
**Medium severity:** Denial of service, circuit-level traffic correlation.
