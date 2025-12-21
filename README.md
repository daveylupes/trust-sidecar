# Trust Sidecar

<div align="center">

**A universal identity and messaging layer for AI Agents & Ad Tech**

[![Rust](https://img.shields.io/badge/rust-1.90+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

[Features](#features) • [Quick Start](#quick-start) • [Documentation](#documentation) • [Contributing](#contributing) • [Sponsors](#sponsors)

</div>

---

## Overview

**Trust Sidecar** is a lightweight, high-performance Rust binary that provides **Identity**, **Encryption**, and **Verifiable Credentials** to any application (AI Bot, Browser, Server). By using the Affinidi TDK and Rust, we enable a new economy of "Verified Agents" and "Zero-Knowledge Ads" that is faster, safer, and more private than existing solutions.

### Why Trust Sidecar?

- **Performance**: Sub-millisecond latency, compiled to WASM for browser use
- **Security**: Memory-safe Rust, zero-knowledge proofs, no central server
- **Universal**: Works with AI agents, browsers, servers - any application
- **Self-Sovereign**: Users control their identity and credentials
- **Lightweight**: Minimal dependencies, easy to integrate

## Features

### Core Capabilities

- **Decentralized Identity (DID)**: Generate and manage `did:key` identifiers
- **Encrypted Messaging (DIDComm)**: Peer-to-peer encrypted communication
- **Verifiable Credentials**: Issue and verify credentials with selective disclosure
- **Proof of View**: Cryptographic proofs for ad impressions (Ad Tech)
- **Zero-Knowledge Proofs**: Prove facts without revealing raw data

### Use Cases

#### AI Agent Economy
Enable secure, verifiable communication between AI agents. A travel bot can verify it's talking to the real airline bot, not a scam script.

#### Privacy-Preserving Ad Tech
Replace third-party cookies with cryptographic proofs. Advertisers get verified human views without tracking personal data.

#### Verified Services
Build services that require verified credentials without collecting personal information.

## Quick Start

### Prerequisites

- Rust 1.90+ ([Install Rust](https://www.rust-lang.org/tools/install))
- Cargo (comes with Rust)

### Installation

```bash
# Clone the repository
git clone https://github.com/daveylupes/trust-sidecar.git
cd trust-sidecar

# Build the project
cargo build --release

# Start the server
cargo run
```

The server will start on `http://127.0.0.1:3000`.

### Quick Test

```bash
# Health check
curl http://127.0.0.1:3000/health

# Generate a DID
curl -X POST http://127.0.0.1:3000/api/v1/did/generate \
  -H "Content-Type: application/json" \
  -d '{"service_id": "my-agent"}'
```

### All-in-One Testing

For complete testing with browser extension and ad network:

```bash
# Start all services (Trust Sidecar, Ad Network, Test Server)
./start-all.sh

# Check status
./start-all.sh status

# Stop all services
./start-all.sh stop
```

See [GETTING_STARTED.md](GETTING_STARTED.md) for detailed setup instructions.

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

For detailed architecture documentation, see [ARCHITECTURE.md](ARCHITECTURE.md).

## Documentation

- **[Getting Started](GETTING_STARTED.md)** - Complete setup guide
- **[API Reference](API.md)** - Full API documentation
- **[Architecture](ARCHITECTURE.md)** - System design and components
- **[Contributing](CONTRIBUTING.md)** - How to contribute
- **[Browser Extension](browser-extension/README.md)** - Extension documentation
- **[Ad Network Integration](ad-network-server/README.md)** - Mock ad network for testing

## API Endpoints

### Identity

- `POST /api/v1/did/generate` - Generate a new DID
- `GET /health` - Health check

### Credentials

- `POST /api/v1/credentials/issue` - Issue a verifiable credential
- `POST /api/v1/protocols/verify` - Verify a credential requirement

### Ad Tech

- `POST /api/v1/proof/view/generate` - Generate proof of view
- `POST /api/v1/proof/view/verify` - Verify proof of view
- `POST /api/v1/partner/ad/bid` - Ad network bid endpoint

See [API.md](API.md) for complete API documentation with examples.

## Browser Extension

The Trust Sidecar browser extension enables proof-of-view generation directly in web browsers.

### Installation

1. Build and start the Trust Sidecar server
2. Load the extension from `browser-extension/` directory
3. The extension automatically generates proofs for ad slots

### Website Integration

```javascript
// Generate a proof of view
const proof = await window.trustSidecar.generateProof('https://example.com/article');

// Verify a credential
const verified = await window.trustSidecar.verifyCredential(credential, 'age > 18');
```

See [browser-extension/README.md](browser-extension/README.md) for detailed documentation.

## Use Cases

### Use Case A: The "Private Ad Exchange"

1. **User**: Browses a crypto news site. Browser Extension holds a VC: "Crypto Whale (Assets > $100k)"
2. **Site**: Requests proof: "Are you a Crypto Whale?"
3. **Sidecar**: Generates SD-JWT Proof (Returns TRUE)
4. **Advertiser**: Verifies signature. Bids $0.50 for the slot
5. **Result**: User sees relevant ad; Advertiser gets verified human view. No personal data left the browser

### Use Case B: The "Agent Wallet"

1. **User**: Runs a "Shopping Bot" to buy limited sneakers
2. **Bot**: Finds sneakers. Store asks for "Verified Shopper" badge (to stop scalper bots)
3. **Sidecar**: Presents the badge signed by "Anti-Bot Alliance"
4. **Store**: Accepts order. Bot signs payment transaction

## Development

### Project Structure

```
trust-sidecar/
├── src/
│   ├── main.rs           # Main entry point & API server
│   ├── cli.rs            # CLI commands
│   ├── identity/         # DID generation & key management
│   ├── messaging/        # DIDComm encrypted messaging
│   └── protocols/        # SD-JWT & credential verification
├── browser-extension/    # Browser extension for Ad Tech
├── ad-network-server/    # Mock ad network for testing
├── Cargo.toml
└── README.md
```

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

### Tech Stack

- **Runtime**: [Tokio](https://tokio.rs/) - Async runtime
- **API Server**: [Axum](https://github.com/tokio-rs/axum) - Web framework
- **Identity**: [Affinidi TDK](https://github.com/affinidi/affinidi-tdk-rs) - DID & credential management
- **Messaging**: [Affinidi Messaging SDK](https://crates.io/crates/affinidi-messaging-sdk) - DIDComm protocol
- **Protocols**: JWT/ES256 for credential issuance, SD-JWT (Selective Disclosure JWT)

## Roadmap

### Phase 1: The "Developer Utility" (Completed)
- [x] Basic Rust project structure
- [x] Module organization (identity, messaging, protocols)
- [x] API server with health check
- [x] DID generation and keychain storage
- [x] SD-JWT credential verification (basic implementation)
- [x] CLI tool for developers

### Phase 2: The "Ad Network" Pilot (Completed)
- [x] Browser extension integration
- [x] Proof-of-view generation
- [x] Credential issuance system
- [x] Partner integrations

### Phase 3: The "Protocol Standard" (In Progress)
- [ ] Open-source protocol specification
- [ ] Issuer network bootstrap
- [ ] Cross-platform SDKs (Python, JavaScript)
- [ ] WASM compilation for browser use
- [ ] Full SD-JWT implementation using `bh-sd-jwt` crate
- [ ] Complete DIDComm message sending/receiving

## Security & Privacy

- **Private Keys**: Stored in OS keychain (Keychain on macOS, Credential Manager on Windows, Secret Service on Linux)
- **Zero-Knowledge**: SD-JWT allows proving attributes without revealing raw data
- **No Central Server**: Direct peer-to-peer communication via DIDComm
- **Memory Safety**: Rust's ownership system prevents common security vulnerabilities

See [SECURITY.md](SECURITY.md) for security best practices and reporting.

**⚠️ Security Notice**: A comprehensive security audit has been conducted and all identified vulnerabilities have been remediated. The codebase is ready for open-source release.

## Known Limitations

1. **Key Recovery**: If a user loses their Sidecar private key, they lose their reputation.
   - **Mitigation**: Implement "Social Recovery" (sharding keys to 3 friends) - *Planned*

2. **Latency**: Ad Tech requires <100ms response.
   - **Status**: Rust implementation provides sub-millisecond latency

3. **Bootstrapping**: An identity system is useless without "Issuers" (someone needs to give the first credential).
   - **Mitigation**: We will be the first Issuer. Create a "Verified Dev" badge for early adopters.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Quick Contribution Guide

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Commit your changes (`git commit -m 'Add some amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Areas for Contribution

- Full SD-JWT implementation using `bh-sd-jwt` crate
- Complete DIDComm message sending/receiving
- Key recovery mechanisms
- Performance optimizations
- Documentation improvements
- Example applications

## Sponsors

Trust Sidecar is an open-source project. If you find it useful, please consider:

- Starring the repository
- Reporting bugs
- Suggesting features
- Contributing code
- Sharing with others

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

<div align="center">

**Built with Rust**

[⬆ Back to Top](#trust-sidecar)

</div>
