# Trust Sidecar Architecture

This document describes the architecture, design decisions, and component structure of Trust Sidecar.

## Overview

Trust Sidecar is designed as a lightweight, high-performance sidecar that provides identity, messaging, and credential services to any application. It follows a modular architecture with clear separation of concerns.

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  AI Agent   │  │   Browser    │  │   Server    │      │
│  │             │  │  Extension   │  │             │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
│         │                  │                  │               │
│         └──────────────────┼──────────────────┘               │
│                            │                                   │
│                            │ HTTP API                          │
└────────────────────────────┼───────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                  Trust Sidecar Core                          │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              API Server (Axum)                        │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐          │  │
│  │  │ Identity │  │ Messaging │  │Protocols │          │  │
│  │  │ Manager  │  │ Manager   │  │ Manager  │          │  │
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘          │  │
│  └───────┼──────────────┼─────────────┼─────────────────┘  │
└──────────┼──────────────┼─────────────┼─────────────────────┘
           │              │             │
           ▼              ▼             ▼
    ┌──────────┐   ┌──────────┐   ┌──────────┐
    │ Affinidi │   │ Affinidi │   │  JWT/    │
    │   TDK    │   │ Messaging│   │ SD-JWT   │
    │          │   │   SDK    │   │          │
    └──────────┘   └──────────┘   └──────────┘
```

## Core Components

### 1. Identity Manager (`src/identity/mod.rs`)

**Purpose**: Manages Decentralized Identifiers (DIDs) and key storage.

**Responsibilities**:
- Generate new DIDs using `did:key` method
- Store and retrieve private keys from OS keychain
- Manage DID-to-service mappings
- Cache credentials for performance

**Key Features**:
- Uses Affinidi TDK for DID generation
- OS keychain integration (Keychain/Credential Manager/Secret Service)
- In-memory caching for performance
- P256 key generation for optimal security

**Data Flow**:
```
Application → IdentityManager → Affinidi TDK → OS Keychain
```

### 2. Messaging Manager (`src/messaging/mod.rs`)

**Purpose**: Handles encrypted peer-to-peer messaging using DIDComm.

**Responsibilities**:
- Initialize ATM (Affinidi Trust Messaging) engine
- Send encrypted messages to recipient DIDs
- Receive and decrypt incoming messages
- Manage messaging profiles

**Current Status**:
- ATM initialization (completed)
- Message sending (placeholder, planned)
- Message receiving (placeholder, planned)

**Future Implementation**:
- Full DIDComm protocol support
- Message routing and delivery
- Profile management

### 3. Protocol Manager (`src/protocols/`)

**Purpose**: Handles verifiable credentials, zero-knowledge proofs, selective disclosure, and agent authorization.

Split across submodules, all as `impl ProtocolManager` blocks on the same shared struct:

- **`mod.rs`** — core `ProtocolManager`, `ProofOfView`, JWT credential issue/verify (ES256), the string-based `verify_requirement` parser, and the shared `register_proof_hash_if_fresh` replay-protection registry used by every hash-based proof type.
- **`credentials.rs`** — `TypedCredential<T>` and `issue_typed_credential`/`verify_typed_credential`: a thin, generic wrapper around the core JWT issue/verify functions so new credential "types" (below) don't need their own signing/verification code path. As a descendant module of `protocols`, it can see the private `CredentialClaims` struct's fields without any visibility changes.
- **`agent_auth.rs`** — `SpendAuthorizationClaims`, `ProposedTransaction`, `SpendAuthorizationCheckResult`, and `check_transaction_against_authorization` (a pure, synchronous constraint checker). See [Agent Authorization](#agent-authorization) below.
- **`proof_of_action.rs`** — `ProofOfAction`: generalizes `ProofOfView` from "a viewer saw this content" to "an agent performed this action, under this authorization, at this time." A parallel struct, not a replacement — `ad-network-server` depends on `ProofOfView`'s exact current shape.

**Responsibilities**:
- Issue verifiable credentials (JWT-based)
- Verify credential requirements
- Generate/verify proof of view for Ad Tech
- Generate/verify proof of action for agent-executed actions
- Issue/verify spend-authorization credentials and check transactions against them
- Selective disclosure proofs (SD-JWT) — planned, not yet implemented

**Key Features**:
- JWT credential issuance (ES256 algorithm)
- Basic requirement verification (e.g., "age > 18")
- SHA-256 proof hashing with age-based replay-registry eviction (entries are evicted individually by age, not wiped wholesale once the registry grows large)
- Timestamp validation

**Future Enhancements**:
- Full SD-JWT implementation using `bh-sd-jwt` crate
- Zero-knowledge proof generation
- Advanced requirement parsing
- Agent-commerce protocol adapters (see [Agent Commerce Landscape](#agent-commerce-landscape))

### Agent Authorization

Lets a principal (human/user DID) grant an agent (agent DID) a constrained, standing authorization to spend on their behalf:

```rust
pub struct SpendAuthorizationClaims {
    pub max_transaction_amount: f64,   // per-transaction cap, not cumulative
    pub currency: String,
    pub merchant_allowlist: Option<Vec<String>>,
    pub category_allowlist: Option<Vec<String>>,
    pub valid_from: i64,               // business-authorization window,
    pub valid_until: i64,              // independent of the JWT's own iat/exp
    pub nonce: String,                 // grant identifier, not single-use
}
```

`principal_did`/`agent_did` are deliberately *not* repeated inside these claims — they're already carried by the wrapping JWT's `iss`/`sub`, to avoid a spoofing surface if the two ever disagreed.

**Issuance vs. verification split** (same security posture as credential issuance in general):

```
Principal's private key (PEM file)
          │
          ▼
   CLI: issue-spend-authorization   ──never transmitted──►  (stays local)
          │
          ▼
    Signed JWT credential
          │
          ▼
   HTTP: POST /api/v1/agent-auth/verify-transaction
   (needs only the credential + principal's PUBLIC key + a proposed transaction)
          │
          ▼
   check_transaction_against_authorization()
   (amount, currency, merchant/category allowlist, validity window)
          │
          ▼
   { authorized: bool, reasons: [...] }
```

Issuing requires the principal's private key and is therefore CLI-only — the same reasoning as [`/api/v1/credentials/issue`](API.md#issue-credential) being disabled over HTTP. Verification needs only a public key and a signed credential, so it's safe to expose as an HTTP endpoint.

### Agent Commerce Landscape

Trust Sidecar's agent-authorization work is positioned as an underlying identity/authorization/evidence layer, not a competitor to emerging agent-commerce protocols (Google AP2, the OpenAI/Stripe Agentic Commerce Protocol, Visa's Trusted Agent Protocol, Mastercard Agent Pay, Coinbase's x402, and others). Its existing primitives — `did:key` identity, ES256 JWT verifiable credentials, SHA-256 proof hashing — line up reasonably well with what several of these protocols expect (e.g. AP2's mandates are SD-JWT-based and explicitly require ECDSA-family signing, not deterministic Ed25519). The near-term plan is to build adapters from Trust Sidecar's own credential/proof types into these external protocols' wire formats, starting with AP2, rather than reimplementing checkout/payment flows directly. None of that adapter work exists yet — see the Phase 4 roadmap in [README.md](README.md#roadmap).

## API Server

### Architecture

The API server uses [Axum](https://github.com/tokio-rs/axum) for high-performance async HTTP handling.

**Request Flow**:
```
HTTP Request → Axum Router → Handler → Manager → Response
```

### Endpoints

#### Identity Endpoints
- `POST /api/v1/did/generate` - Generate new DID
- `GET /health` - Health check

#### Credential Endpoints
- `POST /api/v1/credentials/issue` - Issue verifiable credential
- `POST /api/v1/protocols/verify` - Verify credential requirement

#### Agent Authorization Endpoints
- `POST /api/v1/agent-auth/verify-transaction` - Check a proposed transaction against a spend-authorization credential
- `POST /api/v1/proof/action/generate` - Generate proof of action
- `POST /api/v1/proof/action/verify` - Verify proof of action

#### Ad Tech Endpoints
- `POST /api/v1/proof/view/generate` - Generate proof of view
- `POST /api/v1/proof/view/verify` - Verify proof of view
- `POST /api/v1/partner/ad/bid` - Ad network bid endpoint

### State Management

The API server uses Axum's `State` extractor to share application state:

```rust
struct AppState {
    identity: Arc<IdentityManager>,
    messaging: Arc<MessagingManager>,
    protocols: Arc<ProtocolManager>,
}
```

This ensures thread-safe access to managers across all handlers.

## Browser Extension Architecture

### Components

1. **Background Service Worker** (`background.js`)
   - Communicates with Trust Sidecar API
   - Manages browser DID storage
   - Handles proof generation requests

2. **Content Script** (`content.js`)
   - Runs in isolated world
   - Detects ad slots on pages
   - Generates proofs for ad slots
   - Dispatches events to page context

3. **Injected Script** (`injected.js`)
   - Runs in page context
   - Exposes `window.trustSidecar` API
   - Bridges page and content script via `postMessage`

4. **Integration Script** (`ad-network-integration.js`)
   - Runs in page context
   - Listens for proof generation events
   - Sends bid requests to ad networks
   - Handles bid responses

### Communication Flow

```
Page Context (injected.js)
    ↕ postMessage
Content Script (content.js)
    ↕ chrome.runtime.sendMessage
Background Worker (background.js)
    ↕ fetch
Trust Sidecar API
```

## Data Structures

### Proof of View

```rust
pub struct ProofOfView {
    pub viewer_did: String,        // DID of the viewer
    pub content_id: String,         // URL or identifier
    pub timestamp: i64,             // Unix timestamp
    pub proof_hash: String,         // SHA-256 hash
    pub credential_proof: Option<String>, // Optional credential proof
}
```

### Credential Claims

```rust
struct CredentialClaims {
    iss: String,      // Issuer DID
    sub: String,      // Subject DID
    iat: i64,         // Issued at
    exp: i64,         // Expiration
    vc_type: String,  // Credential type
    claims: Value,    // Additional claims
}
```

`CredentialClaims` is intentionally private; `protocols::credentials::TypedCredential<T>` (below) is the public-facing way to get typed claims back out of a verified credential.

### Typed Credential

```rust
pub struct TypedCredential<T> {
    pub issuer_did: String,
    pub subject_did: String,
    pub issued_at: i64,
    pub expires_at: i64,
    pub credential_type: String,
    pub claims: T,
}
```

### Spend Authorization Claims

```rust
pub struct SpendAuthorizationClaims {
    pub max_transaction_amount: f64,          // per-transaction cap
    pub currency: String,                      // ISO 4217, e.g. "USD"
    pub merchant_allowlist: Option<Vec<String>>,
    pub category_allowlist: Option<Vec<String>>,
    pub valid_from: i64,
    pub valid_until: i64,
    pub nonce: String,                         // grant identifier, not single-use
}
```

### Proof of Action

```rust
pub struct ProofOfAction {
    pub actor_did: String,
    pub action_id: String,
    pub authorization_ref: Option<String>,     // e.g. a SpendAuthorization's nonce
    pub timestamp: i64,
    pub proof_hash: String,
    pub credential_proof: Option<String>,
}
```

## Security Considerations

### Key Storage

- Private keys stored in OS keychain
- Never exposed in memory longer than necessary
- Keys are encrypted at rest by OS

### Proof Verification

- SHA-256 hashing prevents tampering
- Timestamp validation + a replay-hash registry (evicted by age, not wholesale-cleared once large) prevents replay attacks
- Cryptographic signatures for credentials

### Rate Limiting

Per-IP rate limiting is enforced via [`tower_governor`](https://crates.io/crates/tower_governor), sized from the `RATE_LIMIT_REQUESTS`/`RATE_LIMIT_DURATION` constants in `src/security.rs`, in addition to the `ConcurrencyLimitLayer` (a cap on concurrent in-flight requests — not a substitute for rate limiting on its own). Note: `tower_governor` versions ≥0.5 depend on `axum 0.8`, which is incompatible with this project's `axum 0.7` — the pinned version (`0.4`) is the one that actually resolves to a single `axum` version in `Cargo.lock`; check this again before bumping either dependency.

### Zero-Knowledge

- SD-JWT enables proving facts without revealing data
- Selective disclosure for privacy-preserving verification

## Performance

### Optimizations

- **Async/Await**: All I/O operations are async
- **Arc Sharing**: Managers shared across handlers via Arc
- **Caching**: DID and credential caching in memory
- **Rust Performance**: Zero-cost abstractions, no GC

### Benchmarks

- DID generation: <10ms
- Proof generation: <1ms
- Credential verification: <5ms
- API response time: <5ms (excluding network)

## Error Handling

### Strategy

- Use `Result<T, Box<dyn Error>>` for fallible operations
- Log errors with `tracing` crate
- Return appropriate HTTP status codes
- Provide clear error messages

### Example

```rust
match operation().await {
    Ok(result) => Ok(Json(result)),
    Err(e) => {
        error!("Operation failed: {}", e);
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
```

## Testing Strategy

### Unit Tests

- Test individual components in isolation
- Mock external dependencies
- Test edge cases and error conditions

### Integration Tests

- Test API endpoints end-to-end
- Test browser extension integration
- Test ad network integration

### Manual Testing

- Use provided test pages
- Use mock ad network server
- Use CLI commands

## Future Architecture Enhancements

### Planned

1. **Full SD-JWT Implementation**
   - Use `bh-sd-jwt` crate
   - Zero-knowledge proof generation
   - Advanced selective disclosure
   - Apply to `SpendAuthorizationClaims` for holder-binding (`cnf`) and selective disclosure

2. **DIDComm Messaging**
   - Complete message sending/receiving
   - Message routing
   - Profile management

3. **WASM Compilation**
   - Compile core to WebAssembly
   - Run in browser without server
   - Reduce latency

4. **Key Recovery**
   - Social recovery mechanism
   - Key sharding
   - Backup strategies

5. **Agent Commerce Authorization** (see [Agent Commerce Landscape](#agent-commerce-landscape))
   - DID-to-public-key resolution (verification currently requires the caller to already have the key)
   - Revocation of an issued spend authorization
   - Cumulative/running spend totals across transactions
   - Adapter(s) for external agent-commerce protocols, starting with Google AP2

## Design Principles

1. **Modularity**: Clear separation of concerns
2. **Performance**: Sub-millisecond latency
3. **Security**: Memory-safe, zero-knowledge proofs
4. **Privacy**: No central server, user-controlled data
5. **Simplicity**: Easy to integrate and use

---

For implementation details, see the source code documentation in each module.
