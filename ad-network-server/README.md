# Mock Ad Network Server

A test server that simulates an ad network for testing Trust Sidecar integration.

## Features

- Receives ad slot bid requests with proof of view
- Verifies proofs with Trust Sidecar API
- Returns bids based on credential requirements and verification status
- Tracks statistics (requests, verified views, bid values)

## Running

```bash
cd ad-network-server
cargo run
```

The server will start on `http://127.0.0.1:8081`

**Prerequisites:** Trust Sidecar must be running on `http://127.0.0.1:3000`

## API Endpoints

### Health Check
```bash
curl http://127.0.0.1:8081/health
```

### Ad Slot Bid Request
```bash
curl -X POST http://127.0.0.1:8081/api/v1/bid \
  -H "Content-Type: application/json" \
  -d '{
    "ad_slot_id": "slot-123",
    "url": "https://example.com/article",
    "required_credentials": ["age > 18", "interest = crypto"],
    "proof_of_view": {
      "viewer_did": "did:key:...",
      "content_id": "https://example.com/article",
      "timestamp": 1234567890,
      "proof_hash": "abc123...",
      "credential_proof": "age > 18"
    }
  }'
```

### Verify Proof
```bash
curl -X POST http://127.0.0.1:8081/api/v1/verify \
  -H "Content-Type: application/json" \
  -d '{
    "proof": {
      "viewer_did": "did:key:...",
      "content_id": "https://example.com/article",
      "timestamp": 1234567890,
      "proof_hash": "abc123...",
      "credential_proof": null
    }
  }'
```

### Statistics
```bash
curl http://127.0.0.1:8081/api/v1/stats
```

## Bid Calculation

Bids are calculated based on:
- Base bid: $0.10
- Verification multiplier: 2.0x if verified, 0.5x if not
- Credential multiplier: +10% per credential requirement

Example:
- Verified proof with 2 credentials: $0.10 × 2.0 × 1.2 = $0.24
- Unverified proof: $0.10 × 0.5 = $0.05
