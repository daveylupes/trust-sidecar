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
- **Agent Spend Authorization**: Let a human (principal) grant an AI agent a constrained, verifiable authorization to spend on their behalf — per-transaction limits, merchant/category allowlists, a validity window — checkable by anyone with just the principal's public key
- **Proof of View**: Cryptographic proofs for ad impressions (Ad Tech)
- **Proof of Action**: Cryptographic proofs that an agent performed a specific action under a specific authorization
- **Zero-Knowledge Proofs**: Prove facts without revealing raw data

### Use Cases

#### AI Agent Economy
Enable secure, verifiable communication between AI agents. A travel bot can verify it's talking to the real airline bot, not a scam script.

#### Agent Commerce Authorization
Let an AI shopping/booking agent act on a person's behalf without handing it unlimited spending power. The person signs a `SpendAuthorization` credential (max amount per transaction, allowed merchants/categories, a time window); any merchant or payment provider can verify a proposed transaction against it using only the person's public key, without contacting Trust Sidecar or the person at transaction time.

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

- `POST /api/v1/credentials/issue` - Issue a verifiable credential (currently disabled — see [API.md](API.md#issue-credential); use the CLI instead)
- `POST /api/v1/protocols/verify` - Verify a credential requirement

### Agent Authorization

- `POST /api/v1/agent-auth/verify-transaction` - Check a proposed transaction against a signed spend-authorization credential
- `POST /api/v1/proof/action/generate` - Generate proof that an agent performed an action
- `POST /api/v1/proof/action/verify` - Verify proof of action

Issuing a spend authorization is CLI-only (`cargo run -- issue-spend-authorization ...`), same reasoning as credential issuance: the private key never goes over HTTP. Verification only needs a public key, so it's exposed over HTTP.

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
│   ├── main.rs               # Main entry point & API server
│   ├── cli.rs                # CLI commands
│   ├── security.rs           # Input validation, rate-limit config, error sanitization
│   ├── identity/              # DID generation & key management
│   ├── messaging/              # DIDComm encrypted messaging
│   └── protocols/               # Credentials, proofs, and agent authorization
│       ├── mod.rs                 # ProtocolManager, ProofOfView, JWT issue/verify
│       ├── credentials.rs         # Typed credential wrapper (TypedCredential<T>)
│       ├── agent_auth.rs          # SpendAuthorization credentials & transaction checks
│       └── proof_of_action.rs     # ProofOfAction (generalized proof-of-view)
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

### Phase 4: The "Agent Commerce Authorization" Layer (Phase 1 Completed)

The strategic bet here: rather than building another checkout/payment protocol (there's no shortage of those — see [ARCHITECTURE.md](ARCHITECTURE.md#agent-commerce-landscape)), Trust Sidecar aims to be the underlying agent-authorization and evidence layer those protocols can sit on top of. Its existing DID + ES256 JWT + hash-proof primitives are a natural fit for that role.

- [x] `SpendAuthorizationClaims` credential: principal grants agent a constrained, standing spend authorization (per-transaction cap, merchant/category allowlists, validity window)
- [x] Structured transaction-constraint checking (`check_transaction_against_authorization`), not string-parsed like the older `verify_requirement`
- [x] `ProofOfAction`: generalized proof-of-view for "agent X performed action Y under authorization Z"
- [x] CLI issuance (`issue-spend-authorization`), safe HTTP verification (`/api/v1/agent-auth/verify-transaction`)
- [ ] SD-JWT selective disclosure / holder-binding for these credentials
- [ ] DID-to-public-key resolution (verification currently requires the caller to already have the issuer's public key, same limitation `verify_credential` has today)
- [ ] Revocation of an issued spend authorization
- [ ] Cumulative/running spend totals across multiple transactions (currently only a per-transaction cap)
- [ ] Adapter(s) for external agent-commerce protocols (Google AP2 is the most natural first target given its SD-JWT-based mandates and ES256-friendly signing; Visa Trusted Agent Protocol / RFC 9421 and others may follow)

## Security & Privacy

- **Private Keys**: Stored in OS keychain (Keychain on macOS, Credential Manager on Windows, Secret Service on Linux)
- **Zero-Knowledge**: SD-JWT allows proving attributes without revealing raw data
- **No Central Server**: Direct peer-to-peer communication via DIDComm
- **Memory Safety**: Rust's ownership system prevents common security vulnerabilities

See [SECURITY.md](SECURITY.md) for security best practices and reporting.

**Security status**: This codebase has gone through an internal code review; known critical and high-severity issues identified during that review have been fixed. This has **not** been through an external, third-party security audit. If you're evaluating this for a security-sensitive use case, we encourage independent review rather than relying on this line alone.

## Known Limitations

1. **Key Recovery**: If a user loses their Sidecar private key, they lose their reputation.
   - **Mitigation**: Implement "Social Recovery" (sharding keys to 3 friends) - *Planned*

2. **Latency**: Ad Tech requires <100ms response.
   - **Status**: Rust implementation provides sub-millisecond latency

3. **Bootstrapping**: An identity system is useless without "Issuers" (someone needs to give the first credential).
   - **Mitigation**: We will be the first Issuer. Create a "Verified Dev" badge for early adopters.

4. **No DID-to-key resolution**: verifying a credential or a spend authorization currently requires the caller to already have the issuer's public key out-of-band — there's no lookup from a DID to its key material yet.
   - **Mitigation**: *Planned* — see the Phase 4 roadmap above.

5. **Spend authorizations don't revoke or accumulate**: a `SpendAuthorization` credential enforces a per-transaction cap, not a running total, and there's no way to invalidate one before it naturally expires.
   - **Mitigation**: *Planned* — revocation and cumulative-limit tracking are open roadmap items, not implemented yet. Don't issue an authorization you wouldn't be comfortable having valid for its full stated window.

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
- DID-to-public-key resolution
- Spend-authorization revocation and cumulative spend tracking
- Agent-commerce protocol adapters (Google AP2, Visa Trusted Agent Protocol, etc. — see the Phase 4 roadmap above)
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
