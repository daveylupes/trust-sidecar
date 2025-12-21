# Getting Started with Trust Sidecar

This guide will help you get Trust Sidecar up and running quickly.

## Prerequisites

### Required

- **Rust 1.90+**: [Install Rust](https://www.rust-lang.org/tools/install)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Cargo**: Comes with Rust installation

### Optional (for full testing)

- **Chrome/Edge Browser**: For browser extension testing
- **Python 3**: For test server (usually pre-installed on Linux/macOS)
- **curl**: For API testing (usually pre-installed)

## Installation

### Step 1: Clone the Repository

```bash
git clone https://github.com/daveylupes/trust-sidecar.git
cd trust-sidecar
```

### Step 2: Build the Project

```bash
# Development build
cargo build

# Or release build (optimized, recommended for production)
cargo build --release
```

### Step 3: Verify Installation

```bash
# Run the health check
cargo run -- --help

# You should see the CLI help menu
```

## Quick Start: Running the Server

### Basic Server

```bash
# Start the server (default: http://127.0.0.1:3000)
cargo run

# Or with custom host/port
cargo run -- serve --host 0.0.0.0 --port 3000
```

### Verify Server is Running

```bash
# In another terminal
curl http://127.0.0.1:3000/health

# Expected response:
# {"status":"ok","service":"trust-sidecar","version":"0.1.0"}
```

## Complete Testing Setup

For full testing with browser extension and ad network integration:

### Option 1: All-in-One Script (Recommended)

```bash
# Start all services (Trust Sidecar, Ad Network, Test Server)
./start-all.sh

# Check status
./start-all.sh status

# Stop all services
./start-all.sh stop
```

### Option 2: Manual Setup

#### Terminal 1: Trust Sidecar Server

```bash
cargo run
```

#### Terminal 2: Ad Network Server

```bash
cd ad-network-server
cargo run
```

#### Terminal 3: Test Server

```bash
cd browser-extension
./start-test-server.sh
```

## Your First API Call

### Generate a DID

```bash
curl -X POST http://127.0.0.1:3000/api/v1/did/generate \
  -H "Content-Type: application/json" \
  -d '{"service_id": "my-first-agent"}'
```

**Response:**
```json
{
  "did": "did:key:z6Mk..."
}
```

### Generate a Proof of View

```bash
curl -X POST http://127.0.0.1:3000/api/v1/proof/view/generate \
  -H "Content-Type: application/json" \
  -d '{
    "viewer_did": "did:key:z6Mk...",
    "content_id": "https://example.com/article",
    "credential_proof": null
  }'
```

**Response:**
```json
{
  "proof": {
    "viewer_did": "did:key:z6Mk...",
    "content_id": "https://example.com/article",
    "timestamp": 1234567890,
    "proof_hash": "abc123...",
    "credential_proof": null
  }
}
```

## Browser Extension Setup

### Step 1: Ensure Server is Running

```bash
# Start Trust Sidecar server
cargo run
```

### Step 2: Load Extension

1. Open Chrome/Edge
2. Navigate to `chrome://extensions/` (or `edge://extensions/`)
3. Enable **Developer mode** (toggle in top-right)
4. Click **"Load unpacked"**
5. Select the `browser-extension/` directory

### Step 3: Verify Extension

1. Look for the Trust Sidecar icon in your toolbar
2. Click the icon to see connection status
3. Open `http://localhost:8080/test-page.html` (if test server is running)
4. Click "Check Status" - should show "Extension loaded and working!"

See [browser-extension/README.md](browser-extension/README.md) for detailed extension documentation.

## CLI Usage

### Generate a DID

```bash
cargo run -- generate-did --service-id my-agent
```

### Verify a Credential

```bash
cargo run -- verify \
  --credential '{"age": 25}' \
  --requirement "age > 18"
```

### Issue a Credential

```bash
# First, generate an issuer DID and key
cargo run -- generate-did --service-id issuer

# Then issue a credential (see API.md for key format)
cargo run -- issue-credential \
  --subject-did "did:key:..." \
  --claims '{"age": 25, "interest": "crypto"}' \
  --issuer-did "did:key:..." \
  --issuer-key /path/to/key.pem \
  --credential-type "VerifiedUser"
```

### Generate Proof of View

```bash
cargo run -- generate-proof-of-view \
  --viewer-did "did:key:..." \
  --content-id "https://example.com/article"
```

## Testing the Ad Network Integration

### Step 1: Start All Services

```bash
./start-all.sh
```

### Step 2: Open Demo Page

1. Open browser with extension loaded
2. Navigate to `http://localhost:8080/ad-network-demo.html`
3. You should see:
   - Extension status: Available
   - Ad Network status: ok
   - Three ad slots that automatically generate proofs and receive bids

### Step 3: Check Statistics

Click the "Statistics" button to see:
- Total requests
- Verified views
- Average bid value

## Troubleshooting

### Server Won't Start

**Port already in use:**
```bash
# Find process using port 3000
lsof -i :3000

# Kill the process
kill -9 <PID>
```

### Extension Not Detected

1. Check extension is loaded: `chrome://extensions/`
2. Check extension is enabled (toggle ON)
3. Reload the extension
4. Refresh the test page
5. Check browser console (F12) for errors

### API Calls Failing

1. Verify server is running: `curl http://127.0.0.1:3000/health`
2. Check server logs for errors
3. Verify request format matches API documentation

### Build Errors

**Rust version too old:**
```bash
rustup update
```

**Missing dependencies:**
```bash
cargo clean
cargo build
```

## Next Steps

- Read the [API Documentation](API.md) for complete API reference
- Explore the [Architecture](ARCHITECTURE.md) to understand the system design
- Check out [Contributing Guidelines](CONTRIBUTING.md) to start contributing
- Review [Browser Extension Documentation](browser-extension/README.md) for extension details

## Getting Help

- **Documentation**: Check the docs in this repository
- **Issues**: [GitHub Issues](https://github.com/daveylupes/trust-sidecar/issues)
- **Discussions**: [GitHub Discussions](https://github.com/daveylupes/trust-sidecar/discussions)
- **Contact**: [@daveylupes](https://x.com/daveylupes) on X (Twitter)

---

**Ready to build?** Check out the [Contributing Guide](CONTRIBUTING.md) to start contributing!
