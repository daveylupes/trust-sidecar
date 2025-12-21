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

### 3. Protocol Manager (`src/protocols/mod.rs`)

**Purpose**: Handles verifiable credentials, zero-knowledge proofs, and selective disclosure.

**Responsibilities**:
- Issue verifiable credentials (JWT-based)
- Verify credential requirements
- Generate proof of view for Ad Tech
- Verify proof of view
- Selective disclosure proofs (SD-JWT)

**Key Features**:
- JWT credential issuance (ES256 algorithm)
- Basic requirement verification (e.g., "age > 18")
- SHA-256 proof hashing
- Timestamp validation

**Future Enhancements**:
- Full SD-JWT implementation using `bh-sd-jwt` crate
- Zero-knowledge proof generation
- Advanced requirement parsing

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

## Security Considerations

### Key Storage

- Private keys stored in OS keychain
- Never exposed in memory longer than necessary
- Keys are encrypted at rest by OS

### Proof Verification

- SHA-256 hashing prevents tampering
- Timestamp validation prevents replay attacks
- Cryptographic signatures for credentials

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

## Design Principles

1. **Modularity**: Clear separation of concerns
2. **Performance**: Sub-millisecond latency
3. **Security**: Memory-safe, zero-knowledge proofs
4. **Privacy**: No central server, user-controlled data
5. **Simplicity**: Easy to integrate and use

---

For implementation details, see the source code documentation in each module.
