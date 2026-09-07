//! Proof of Action
//!
//! Generalizes the `ProofOfView` pattern (see `mod.rs`) from "a viewer saw
//! this content" to "an agent performed this action, under this
//! authorization, at this time" — cryptographic evidence for agent-executed
//! actions rather than ad impressions.
//!
//! This is a new, parallel struct rather than a rename/extension of
//! `ProofOfView`: `ad-network-server` hand-declares its own matching
//! `ProofOfView` struct and depends on its current shape, so changing it
//! would silently break that integration at runtime (a separate crate won't
//! fail to compile, it'll just start getting shape mismatches). Both share
//! the same replay-protection registry via `ProtocolManager::register_proof_hash_if_fresh`.
//!
//! Like `ProofOfView`, generating/verifying a proof involves no private key
//! material, so both operations are safe to expose over HTTP.

use super::ProtocolManager;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::error::Error;
use tracing::info;

/// Cryptographic proof that an agent performed a specific action, optionally
/// under a specific authorization (e.g. a `SpendAuthorization`'s `nonce`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOfAction {
    /// DID of the agent that performed the action
    pub actor_did: String,
    /// Identifier/description of the action (e.g. an order id, an endpoint called)
    pub action_id: String,
    /// Reference to the authorization this action was performed under, if any
    /// (e.g. a `SpendAuthorizationClaims::nonce`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorization_ref: Option<String>,
    pub timestamp: i64,
    pub proof_hash: String,
    /// Optional free-form context folded into the hash, kept for parity with `ProofOfView`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_proof: Option<String>,
}

impl ProtocolManager {
    /// Generate a proof that `actor_did` performed `action_id`.
    pub async fn generate_proof_of_action(
        &self,
        actor_did: &str,
        action_id: &str,
        authorization_ref: Option<&str>,
        credential_proof: Option<&str>,
    ) -> Result<ProofOfAction, Box<dyn Error>> {
        info!("Generating proof of action for: {}", action_id);

        let timestamp = Utc::now().timestamp();

        let mut hasher = Sha256::new();
        hasher.update(actor_did.as_bytes());
        hasher.update(action_id.as_bytes());
        if let Some(r) = authorization_ref {
            hasher.update(r.as_bytes());
        }
        hasher.update(timestamp.to_string().as_bytes());
        if let Some(p) = credential_proof {
            hasher.update(p.as_bytes());
        }
        let proof_hash = format!("{:x}", hasher.finalize());

        let proof = ProofOfAction {
            actor_did: actor_did.to_string(),
            action_id: action_id.to_string(),
            authorization_ref: authorization_ref.map(String::from),
            timestamp,
            proof_hash,
            credential_proof: credential_proof.map(String::from),
        };

        info!("Proof of action generated successfully");
        Ok(proof)
    }

    /// Verify a `ProofOfAction`: recompute the hash, then apply the same
    /// replay + timestamp-bounds check used by `verify_proof_of_view`.
    pub async fn verify_proof_of_action(&self, proof: &ProofOfAction) -> Result<bool, Box<dyn Error>> {
        info!("Verifying proof of action...");

        let mut hasher = Sha256::new();
        hasher.update(proof.actor_did.as_bytes());
        hasher.update(proof.action_id.as_bytes());
        if let Some(ref r) = proof.authorization_ref {
            hasher.update(r.as_bytes());
        }
        hasher.update(proof.timestamp.to_string().as_bytes());
        if let Some(ref p) = proof.credential_proof {
            hasher.update(p.as_bytes());
        }
        let computed_hash = format!("{:x}", hasher.finalize());

        if computed_hash != proof.proof_hash {
            return Ok(false);
        }

        if !self
            .register_proof_hash_if_fresh(&proof.proof_hash, proof.timestamp)
            .await
        {
            return Ok(false);
        }

        info!("Proof of action verified successfully");
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_and_verify_round_trip() -> Result<(), Box<dyn Error>> {
        let manager = ProtocolManager::new();
        let proof = manager
            .generate_proof_of_action("did:key:zAgent", "order-123", Some("nonce-abc"), None)
            .await?;

        assert!(manager.verify_proof_of_action(&proof).await?);
        Ok(())
    }

    #[tokio::test]
    async fn test_tampered_hash_rejected() -> Result<(), Box<dyn Error>> {
        let manager = ProtocolManager::new();
        let mut proof = manager
            .generate_proof_of_action("did:key:zAgent", "order-123", None, None)
            .await?;
        proof.proof_hash = "0".repeat(64);

        assert!(!manager.verify_proof_of_action(&proof).await?);
        Ok(())
    }

    #[tokio::test]
    async fn test_replay_rejected() -> Result<(), Box<dyn Error>> {
        let manager = ProtocolManager::new();
        let proof = manager
            .generate_proof_of_action("did:key:zAgent", "order-123", None, None)
            .await?;

        assert!(manager.verify_proof_of_action(&proof).await?);
        // Second verification of the identical proof must be rejected as a replay
        assert!(!manager.verify_proof_of_action(&proof).await?);
        Ok(())
    }

    #[tokio::test]
    async fn test_replay_registry_survives_growth_past_ten_thousand() -> Result<(), Box<dyn Error>> {
        // Regression test for the fixed bug: the old implementation cleared the
        // *entire* replay registry once it grew past 10,000 entries, which meant
        // a previously-used, still-fresh proof would become replayable again
        // just because unrelated traffic pushed the registry over that size.
        let manager = ProtocolManager::new();
        let now = Utc::now().timestamp();

        let recent_hash = "recent-hash";
        assert!(manager.register_proof_hash_if_fresh(recent_hash, now).await);

        // Flood the registry with many other fresh (still-valid) hashes, pushing
        // the total count well past 10,000.
        for i in 0..10_100 {
            let hash = format!("other-hash-{}", i);
            assert!(manager.register_proof_hash_if_fresh(&hash, now).await);
        }

        // The original hash must still be present in the registry (a repeat
        // registration is rejected as a replay) even though the registry grew
        // far past 10,000 entries in the process — it must NOT have been
        // wholesale-cleared.
        assert!(!manager.register_proof_hash_if_fresh(recent_hash, now).await);

        Ok(())
    }
}
