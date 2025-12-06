# Trust Sidecar

> A universal identity and messaging layer for AI Agents & Ad Tech

[![Rust](https://img.shields.io/badge/rust-1.90+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

The **Trust Sidecar** is a lightweight, high-performance Rust binary that provides **Identity**, **Encryption**, and **Verifiable Credentials** to any application (AI Bot, Browser, Server). By using the Affinidi TDK and Rust, we enable a new economy of "Verified Agents" and "Zero-Knowledge Ads" that is faster, safer, and more private than existing solutions.

## The Problem

### For AI Agents (The "Agentic Economy")
- **The Trust Gap**: An AI travel agent cannot pay an AI airline agent because it cannot verify who the other bot is. Is it the real Delta Air Lines bot or a scam script?
- **Platform Lock-in**: Current solutions (Fetch.ai, AutoGPT) are "walled gardens." A Fetch agent cannot easily talk to a LangChain agent securely.
- **Performance**: Python-based trust layers are too slow for high-frequency agent negotiations (e.g., trading bots).

### For Ad Tech (The "Privacy Crisis")
- **Cookie Death**: Third-party cookies are dying (GDPR/Chrome). Advertisers are losing targeting data.
- **Bot Fraud**: Advertisers lose ~$100B/year to fake bot clicks. They have no cryptographic way to prove a "view" came from a human.

## The Solution

A universal, protocol-agnostic "node" that runs alongside any app.

### Core Technology

- **Identity (DID)**: Every agent/user gets a unique `did:key` (Decentralized Identifier)
- **Transport (DIDComm)**: Encrypted, peer-to-peer JSON messaging. No central server reads the messages.
- **Verification (SD-JWT)**: "Selective Disclosure" Credentials. Agents prove facts (e.g., "I am verified by Visa") without revealing raw data.
- **Runtime (Rust)**: Memory-safe, sub-millisecond latency, compiles to WASM (runs in browsers).

## Architecture

```
┌─────────────────────────────────────────┐
│         Your Application                │
│  (AI Bot, Browser Extension, Server)    │
└──────────────┬──────────────────────────┘
               │ HTTP API
               ▼
┌─────────────────────────────────────────┐
│         Trust Sidecar                   │
│  ┌──────────┐  ┌──────────┐  ┌────────┐│
│  │ Identity │  │Messaging │  │Protocol││
│  │  (DID)   │  │(DIDComm) │  │(SD-JWT)││
│  └──────────┘  └──────────┘  └────────┘│
└─────────────────────────────────────────┘
               │
               ▼
        Affinidi TDK & Network
```

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/daveylupes/trust-sidecar.git
cd trust-sidecar

# Build the project
cargo build --release

# Run the sidecar
cargo run
```

The sidecar will start an API server on `http://127.0.0.1:3000`.

### API Endpoints

#### Health Check
```bash
curl http://127.0.0.1:3000/health
```

#### Generate a New DID
```bash
curl -X POST http://127.0.0.1:3000/api/v1/did/generate \
  -H "Content-Type: application/json" \
  -d '{"service_id": "my-agent"}'
```

#### Verify a Credential Requirement
```bash
curl -X POST http://127.0.0.1:3000/api/v1/protocols/verify \
  -H "Content-Type: application/json" \
  -d '{
    "credential": "eyJ...",
    "requirement": "age > 18"
  }'
```

## Use Cases

### Use Case A: The "Private Ad Exchange"

1. **User**: Browses a crypto news site. Browser Extension (Sidecar) holds a VC: "Crypto Whale (Assets > $100k)".
2. **Site**: Requests proof: "Are you a Crypto Whale?"
3. **Sidecar**: Generates SD-JWT Proof (Returns TRUE).
4. **Advertiser**: Verifies signature. Bids $0.50 for the slot.
5. **Result**: User sees relevant ad; Advertiser gets verified human view. No personal data left the browser.

### Use Case B: The "Agent Wallet"

1. **User**: Runs a "Shopping Bot" to buy limited sneakers.
2. **Bot**: Finds sneakers. Store asks for "Verified Shopper" badge (to stop scalper bots).
3. **Sidecar**: Presents the badge signed by "Anti-Bot Alliance."
4. **Store**: Accepts order. Bot signs payment transaction.

## Development

### Project Structure

```
trust-sidecar/
├── src/
│   ├── main.rs           # Main entry point & API server
│   ├── identity/         # DID generation & key management
│   │   └── mod.rs
│   ├── messaging/        # DIDComm encrypted messaging
│   │   └── mod.rs
│   └── protocols/        # SD-JWT & credential verification
│       └── mod.rs
├── Cargo.toml
└── README.md
```

### Tech Stack

- **Runtime**: [Tokio](https://tokio.rs/) - Async runtime
- **API Server**: [Axum](https://github.com/tokio-rs/axum) - Web framework
- **Identity**: [Affinidi TDK](https://github.com/affinidi/affinidi-tdk-rs) - DID & credential management
- **Messaging**: [Affinidi Messaging SDK](https://crates.io/crates/affinidi-messaging-sdk) - DIDComm protocol
- **Protocols**: SD-JWT (Selective Disclosure JWT) - Zero-knowledge proofs

### Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

## Roadmap

### Phase 1: The "Developer Utility" (Months 1-3)
- [x] Basic Rust project structure
- [x] Module organization (identity, messaging, protocols)
- [x] API server with health check
- [x] DID generation and keychain storage
- [ ] DIDComm message sending/receiving
- [x] SD-JWT credential verification (basic implementation)
- [x] CLI tool for developers

### Phase 2: The "Ad Network" Pilot (Months 4-6)
- [ ] Browser extension integration
- [ ] Proof-of-view generation
- [ ] Credential issuance system
- [ ] Partner integrations

### Phase 3: The "Protocol Standard" (Month 6+)
- [ ] Open-source protocol specification
- [ ] Issuer network bootstrap
- [ ] Cross-platform SDKs (Python, JavaScript)
- [ ] WASM compilation for browser use

## Security & Privacy

- **Private Keys**: Stored in OS keychain (Keychain on macOS, Credential Manager on Windows, Secret Service on Linux)
- **Zero-Knowledge**: SD-JWT allows proving attributes without revealing raw data
- **No Central Server**: Direct peer-to-peer communication via DIDComm
- **Memory Safety**: Rust's ownership system prevents common security vulnerabilities

## Known Limitations & Risks

1. **Key Recovery**: If a user loses their Sidecar private key, they lose their reputation.
   - **Mitigation**: Implement "Social Recovery" (sharding keys to 3 friends) - *Planned*

2. **Latency**: Ad Tech requires <100ms response.
   - **Mitigation**: Rust is essential here. Do not use Python for the verification loop.

3. **Bootstrapping**: An identity system is useless without "Issuers" (someone needs to give the first credential).
   - **Mitigation**: We will be the first Issuer. Create a "Verified Dev" badge for early adopters.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Affinidi](https://affinidi.com/) for the TDK and messaging SDK
- The Rust community for excellent crates and tooling
- The DIDComm and SD-JWT protocol communities

## Contact & Support

- **X (Twitter)**: [@daveylupes](https://x.com/daveylupes)
- **Issues**: [GitHub Issues](https://github.com/daveylupes/trust-sidecar/issues)
- **Discussions**: [GitHub Discussions](https://github.com/daveylupes/trust-sidecar/discussions)

---

Built with Rust

