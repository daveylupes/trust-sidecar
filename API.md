# Trust Sidecar API Reference

Complete API documentation for Trust Sidecar.

## Base URL

```
http://127.0.0.1:3000
```

## Authentication

Currently, the API does not require authentication. In production, API keys or OAuth tokens may be required.

## Endpoints

### Health Check

Check if the server is running.

**Endpoint:** `GET /health`

**Response:**
```json
{
  "status": "ok",
  "service": "trust-sidecar",
  "version": "0.1.0"
}
```

**Example:**
```bash
curl http://127.0.0.1:3000/health
```

---

### Generate DID

Generate a new Decentralized Identifier (DID).

**Endpoint:** `POST /api/v1/did/generate`

**Request Body:**
```json
{
  "service_id": "my-agent"  // Optional: Associate DID with a service
}
```

**Response:**
```json
{
  "did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
}
```

**Example:**
```bash
curl -X POST http://127.0.0.1:3000/api/v1/did/generate \
  -H "Content-Type: application/json" \
  -d '{"service_id": "my-agent"}'
```

**Notes:**
- The DID is generated using the `did:key` method
- Private keys are stored in the OS keychain
- If `service_id` is provided, the DID is cached and can be retrieved later

---

### Issue Credential

Issue a verifiable credential (JWT) for a subject.

**Endpoint:** `POST /api/v1/credentials/issue`

**Request Body:**
```json
{
  "subject_did": "did:key:z6Mk...",
  "claims": {
    "age": 25,
    "interest": "crypto",
    "verified": true
  },
  "issuer_did": "did:key:z6Mk...",
  "issuer_private_key": "-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----",
  "credential_type": "VerifiedUser",
  "expiration_days": 365
}
```

**Response:**
```json
{
  "credential": "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Example:**
```bash
curl -X POST http://127.0.0.1:3000/api/v1/credentials/issue \
  -H "Content-Type: application/json" \
  -d '{
    "subject_did": "did:key:z6Mk...",
    "claims": {"age": 25, "interest": "crypto"},
    "issuer_did": "did:key:z6Mk...",
    "issuer_private_key": "-----BEGIN PRIVATE KEY-----\n...",
    "credential_type": "VerifiedUser",
    "expiration_days": 365
  }'
```

**Notes:**
- Uses ES256 (ECDSA P-256) algorithm
- Private key must be in PEM format
- Credential expires after `expiration_days` (default: 365)
- The credential is a signed JWT

---

### Verify Credential Requirement

Verify that a credential satisfies a requirement.

**Endpoint:** `POST /api/v1/protocols/verify`

**Request Body:**
```json
{
  "credential": "{\"age\": 25, \"interest\": \"crypto\"}",
  "requirement": "age > 18"
}
```

**Response:**
```json
{
  "verified": true
}
```

**Example:**
```bash
curl -X POST http://127.0.0.1:3000/api/v1/protocols/verify \
  -H "Content-Type: application/json" \
  -d '{
    "credential": "{\"age\": 25}",
    "requirement": "age > 18"
  }'
```

**Supported Requirements:**
- `field > value` - Greater than (e.g., "age > 18")
- `field = value` - Equality (e.g., "interest = crypto")
- More operators coming soon

**Notes:**
- Currently supports basic JSON credential format
- Full SD-JWT verification coming soon
- Returns `false` if requirement cannot be verified

---

### Generate Proof of View

Generate a cryptographic proof that content was viewed (for Ad Tech).

**Endpoint:** `POST /api/v1/proof/view/generate`

**Request Body:**
```json
{
  "viewer_did": "did:key:z6Mk...",
  "content_id": "https://example.com/article",
  "credential_proof": "age > 18"  // Optional
}
```

**Response:**
```json
{
  "proof": {
    "viewer_did": "did:key:z6Mk...",
    "content_id": "https://example.com/article",
    "timestamp": 1234567890,
    "proof_hash": "05b59f68255c9cd0c941fec3ec459fad531b67546a561e5e73115a0825a56d5b",
    "credential_proof": "age > 18"
  }
}
```

**Example:**
```bash
curl -X POST http://127.0.0.1:3000/api/v1/proof/view/generate \
  -H "Content-Type: application/json" \
  -d '{
    "viewer_did": "did:key:z6Mk...",
    "content_id": "https://example.com/article",
    "credential_proof": null
  }'
```

**Notes:**
- Proof hash is SHA-256 of viewer_did + content_id + timestamp + credential_proof
- Timestamp is Unix epoch seconds
- Proofs expire after 24 hours
- Used by ad networks to verify human views

---

### Verify Proof of View

Verify that a proof of view is valid.

**Endpoint:** `POST /api/v1/proof/view/verify`

**Request Body:**
```json
{
  "proof": {
    "viewer_did": "did:key:z6Mk...",
    "content_id": "https://example.com/article",
    "timestamp": 1234567890,
    "proof_hash": "05b59f68255c9cd0c941fec3ec459fad531b67546a561e5e73115a0825a56d5b",
    "credential_proof": null
  }
}
```

**Response:**
```json
{
  "verified": true
}
```

**Example:**
```bash
curl -X POST http://127.0.0.1:3000/api/v1/proof/view/verify \
  -H "Content-Type: application/json" \
  -d '{
    "proof": {
      "viewer_did": "did:key:z6Mk...",
      "content_id": "https://example.com/article",
      "timestamp": 1234567890,
      "proof_hash": "abc123...",
      "credential_proof": null
    }
  }'
```

**Verification Checks:**
1. Hash matches recomputed hash
2. Timestamp is not older than 24 hours
3. All required fields are present

---

### Ad Network Bid (Partner Integration)

Ad network bid endpoint for partner integrations.

**Endpoint:** `POST /api/v1/partner/ad/bid`

**Request Body:**
```json
{
  "ad_slot_id": "slot-123",
  "required_credentials": ["age > 18", "interest = crypto"],
  "max_bid": 0.50
}
```

**Response:**
```json
{
  "bid": 0.50,
  "proof_required": true,
  "message": "Bid for slot slot-123 with 2 credential requirements"
}
```

**Example:**
```bash
curl -X POST http://127.0.0.1:3000/api/v1/partner/ad/bid \
  -H "Content-Type: application/json" \
  -d '{
    "ad_slot_id": "slot-123",
    "required_credentials": ["age > 18", "interest = crypto"],
    "max_bid": 0.50
  }'
```

**Notes:**
- This is a placeholder endpoint for partner integrations
- In production, this would verify credentials and return appropriate bids
- `proof_required` indicates if proof of view is required

---

## Error Responses

All endpoints may return the following error responses:

### 400 Bad Request

Invalid request format or missing required fields.

```json
{
  "error": "Invalid request: missing required field 'viewer_did'"
}
```

### 500 Internal Server Error

Server error during processing.

```json
{
  "error": "Failed to generate DID: ..."
}
```

**Note:** Error messages may vary. Check server logs for detailed error information.

---

## Rate Limiting

Currently, there are no rate limits. In production, rate limiting may be implemented.

## CORS

CORS is enabled for all origins in development. In production, configure appropriate CORS policies.

---

## Browser Extension API

The browser extension exposes a `window.trustSidecar` API:

### `generateProof(contentId, credentialProof?)`

Generate a proof of view.

```javascript
const proof = await window.trustSidecar.generateProof('https://example.com/article');
```

### `verifyCredential(credential, requirement)`

Verify a credential requirement.

```javascript
const verified = await window.trustSidecar.verifyCredential(
  '{"age": 25}',
  'age > 18'
);
```

### `getDid()`

Get the browser's DID.

```javascript
const did = await window.trustSidecar.getDid();
```

---

## Examples

### Complete Ad Tech Flow

```bash
# 1. Generate a DID for the viewer
VIEWER_DID=$(curl -s -X POST http://127.0.0.1:3000/api/v1/did/generate \
  -H "Content-Type: application/json" \
  -d '{"service_id": "viewer"}' | jq -r '.did')

# 2. Generate a proof of view
PROOF=$(curl -s -X POST http://127.0.0.1:3000/api/v1/proof/view/generate \
  -H "Content-Type: application/json" \
  -d "{
    \"viewer_did\": \"$VIEWER_DID\",
    \"content_id\": \"https://example.com/article\",
    \"credential_proof\": null
  }")

# 3. Verify the proof
curl -X POST http://127.0.0.1:3000/api/v1/proof/view/verify \
  -H "Content-Type: application/json" \
  -d "{\"proof\": $(echo $PROOF | jq '.proof')}"
```

### Credential Issuance Flow

```bash
# 1. Generate issuer DID and key (use CLI)
cargo run -- generate-did --service-id issuer

# 2. Issue a credential
curl -X POST http://127.0.0.1:3000/api/v1/credentials/issue \
  -H "Content-Type: application/json" \
  -d '{
    "subject_did": "did:key:...",
    "claims": {"age": 25, "interest": "crypto"},
    "issuer_did": "did:key:...",
    "issuer_private_key": "-----BEGIN PRIVATE KEY-----\n...",
    "credential_type": "VerifiedUser"
  }'
```

---

For more examples, see the [Getting Started Guide](GETTING_STARTED.md) and test files in the repository.
