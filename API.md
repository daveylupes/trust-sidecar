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

**This endpoint is currently disabled.** It always returns `501 Not Implemented`.

**Endpoint:** `POST /api/v1/credentials/issue`

Credential issuance requires the issuer's private key. Earlier versions of this
endpoint accepted that key directly in the request body (`issuer_private_key`)
— this was removed for security, since it meant private key material was
transmitted over HTTP. The request struct now expects an `issuer_service_id`
instead (to eventually load the key from the OS keychain via TDK), but that
keychain-retrieval integration isn't implemented yet, so the endpoint simply
rejects every request with `501` in the meantime rather than accepting a
request it can't safely fulfill.

**Use the CLI instead**, which reads the issuer's private key from a local
PEM file and never transmits it anywhere:

```bash
cargo run -- issue-credential \
  --subject-did "did:key:z6Mk..." \
  --claims '{"age": 25, "interest": "crypto"}' \
  --issuer-did "did:key:z6Mk..." \
  --issuer-key /path/to/issuer_private_key.pem \
  --credential-type "VerifiedUser" \
  --expiration-days 365
```

**Notes:**
- Uses ES256 (ECDSA P-256) algorithm
- Private key must be in PEM format
- Credential expires after `--expiration-days` (default: 365)
- The credential is a signed JWT
- See [GETTING_STARTED.md](GETTING_STARTED.md#cli-usage) for the full CLI walkthrough

---

### Issue Spend Authorization

Issue a credential letting an agent DID spend on behalf of a principal
(human/user) DID, within stated constraints. **CLI-only**, for the same
reason as credential issuance above: it requires the principal's private key,
which is read from a local PEM file and never transmitted.

```bash
cargo run -- issue-spend-authorization \
  --principal-did "did:key:zPrincipal..." \
  --principal-key /path/to/principal_private_key.pem \
  --agent-did "did:key:zAgent..." \
  --max-amount 50.00 \
  --currency USD \
  --merchants "merchant-123,merchant-456" \
  --categories "groceries" \
  --valid-days 30
```

Prints the signed JWT credential. `--merchants`/`--categories` are optional
comma-separated allowlists (omit for no restriction on that dimension).
`--max-amount` is a **per-transaction** cap on a reusable authorization, not a
single-use grant or a cumulative budget — the same credential can be checked
against many transactions.

**Notes:**
- Uses ES256 (ECDSA P-256), same as other credentials in this crate
- There is no revocation mechanism yet; once issued, a credential is valid
  until its `--valid-until`/`--valid-days` window or JWT `--expiration-days`
  lapses
- No DID-to-public-key resolution exists yet — verification (below) needs the
  principal's public key passed explicitly, the same limitation `verify_credential`
  already has

---

### Verify Spend Authorization / Transaction

Check a proposed transaction against a spend-authorization credential's
constraints. Unlike issuance, this is **safe over HTTP** — it only needs the
signed credential and the issuer's (principal's) *public* key, never private
key material.

**Endpoint:** `POST /api/v1/agent-auth/verify-transaction`

**Request Body:**
```json
{
  "credential": "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9...",
  "issuer_public_key": "-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----",
  "transaction": {
    "amount": 25.00,
    "currency": "USD",
    "merchant_id": "merchant-123",
    "category": "groceries",
    "timestamp": 1234567890
  }
}
```

**Response:**
```json
{
  "authorized": true,
  "reasons": [],
  "agent_did": "did:key:zAgent...",
  "principal_did": "did:key:zPrincipal..."
}
```

If not authorized, `reasons` lists which constraints failed, e.g.
`["amount_exceeds_max", "merchant_not_allowed"]`.

**Example:**
```bash
curl -X POST http://127.0.0.1:3000/api/v1/agent-auth/verify-transaction \
  -H "Content-Type: application/json" \
  -d '{
    "credential": "<JWT from issue-spend-authorization>",
    "issuer_public_key": "-----BEGIN PUBLIC KEY-----\n...",
    "transaction": {
      "amount": 25.00,
      "currency": "USD",
      "merchant_id": "merchant-123",
      "timestamp": 1234567890
    }
  }'
```

Equivalent CLI form (no server required):
```bash
cargo run -- verify-spend-authorization \
  --credential "<JWT>" \
  --issuer-public-key /path/to/principal_public_key.pem \
  --amount 25.00 --currency USD --merchant-id merchant-123
```

---

### Proof of Action

Generalizes [Proof of View](#generate-proof-of-view) from "a viewer saw this
content" to "an agent performed this action, under this authorization, at
this time." Same SHA-256-hash-with-replay-protection design; involves no
private key material, so both operations are safe over HTTP.

**Endpoints:**
- `POST /api/v1/proof/action/generate`
- `POST /api/v1/proof/action/verify`

**Generate — Request Body:**
```json
{
  "actor_did": "did:key:zAgent...",
  "action_id": "order-123",
  "authorization_ref": "nonce-from-spend-authorization",
  "credential_proof": null
}
```

**Generate — Response:**
```json
{
  "proof": {
    "actor_did": "did:key:zAgent...",
    "action_id": "order-123",
    "authorization_ref": "nonce-from-spend-authorization",
    "timestamp": 1234567890,
    "proof_hash": "...",
    "credential_proof": null
  }
}
```

**Verify — Request Body:** `{ "proof": { ...as generated above... } }`
**Verify — Response:** `{ "verified": true }`

**Notes:**
- `authorization_ref` is typically a `SpendAuthorization` credential's nonce,
  linking the action back to the authorization it was performed under
- Proofs expire after 1 hour and cannot be replayed (a second verify of the
  same proof returns `verified: false`), same rules as Proof of View

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
