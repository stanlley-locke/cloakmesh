# CloakMesh Protocol Specification (.cloak)

**Version:** 1.0-Draft
**Status:** Phase 1 (Core Definitions)

## Overview

The CloakMesh protocol provides a decentralized, metadata-resistant communication layer. It uses onion routing to obfuscate the path between a client and a service, and a sharded Kademlia DHT to store service descriptors.

## Addressing

A `.cloak` address is a Bech32-encoded string derived from a service's Ed25519 public key.

**Format:** `<v3>.<base32(identity_pubkey + checksum)>.cloak`
**Example:** `cloak1qy8x3m7n2p5v9k4w6j1r0h8t2f4d.cloak`

## Cryptographic Foundation

- **Identity:** Ed25519 (Signing)
- **Key Exchange:** X25519 + Kyber-768 (Hybrid)
- **Encryption:** ChaCha20-Poly1305
- **Handshake:** Noise_XX_25519_ChaChaPoly_BLAKE2s

## Protocol Flow (The 9 Acts)

1.  **IP Registration:** Service selects Introduction Points (IPs) and establishes circuits.
2.  **Descriptor Publication:** Service signs and publishes its descriptor to the DHT.
3.  **Address Acquisition:** Client obtains the `.cloak` address out-of-band.
4.  **Descriptor Fetch:** Client fetches the descriptor from the DHT via an anonymized circuit.
5.  **Verification:** Client verifies the descriptor's signature and policies.
6.  **RP Establishment:** Client selects a Rendezvous Point (RP) and establishes a circuit.
7.  **Introduction:** Client sends an `INTRODUCE1` message to the Service via an IP.
8.  **Service Connection:** Service connects to the RP via its own circuit.
9.  **Rendezvous:** RP joins the two circuits, enabling end-to-end encrypted communication.

## Message Framing

All messages are framed into fixed-size cells to prevent traffic analysis based on packet size.
- **Default Cell Size:** 512 bytes.
- **Serialization:** Protobuf (gRPC) or CBOR (binary).

## Routing

- **Adaptive Circuits:** 2-6 hops depending on the required security level.
- **Relay Selection:** Weighted by reputation (latency, uptime, bandwidth).

## DHT (Distributed Hash Table)

- **Algorithm:** Kademlia with XOR distance metric.
- **Sharding:** Descriptors are sharded across multiple nodes.
- **Verification:** Merkle-verifiable storage proofs.

---
For detailed implementation details, see `core/src/cloak_protocol/`.
