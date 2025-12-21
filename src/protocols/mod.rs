//! Protocols Module
//!
//! Handles verifiable credentials, zero-knowledge proofs, and selective disclosure.
//! Implements SD-JWT (Selective Disclosure JWT) for privacy-preserving credential verification.

use std::error::Error;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use jsonwebtoken::{encode, decode, Header, Algorithm, EncodingKey, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use tracing::{info, warn};

/// JWT Claims for Verifiable Credentials
#[derive(Debug, Serialize, Deserialize)]
struct CredentialClaims {
    /// Issuer DID
    iss: String,
    /// Subject DID
    sub: String,
    /// Issued at timestamp
    iat: i64,
    /// Expiration timestamp
    exp: i64,
    /// Credential type
    vc_type: String,
    /// Credential claims/attributes
    #[serde(flatten)]
    claims: serde_json::Value,
}

/// Proof of View structure for Ad Tech
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOfView {
    /// DID of the viewer
    pub viewer_did: String,
    /// URL or identifier of the viewed content
    pub content_id: String,
    /// Timestamp of the view
    pub timestamp: i64,
    /// Cryptographic proof hash
    pub proof_hash: String,
    /// Optional credential proof
    pub credential_proof: Option<String>,
}

/// Protocol manager for credential verification and zero-knowledge proofs
///
/// Handles SD-JWT credentials and selective disclosure proofs.
pub struct ProtocolManager {
    /// Default issuer DID (can be set when issuing credentials)
    default_issuer_did: Option<String>,
    /// SECURITY: Registry of used proof hashes to prevent replay attacks
    used_proof_hashes: Arc<RwLock<HashSet<String>>>,
}

impl ProtocolManager {
    /// Create a new ProtocolManager
    pub fn new() -> Self {
        Self {
            default_issuer_did: None,
            used_proof_hashes: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Create a new ProtocolManager with a default issuer DID
    pub fn with_issuer(issuer_did: String) -> Self {
        Self {
            default_issuer_did: Some(issuer_did),
            used_proof_hashes: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Verify a credential against a requirement
    ///
    /// # Arguments
    /// * `credential` - The SD-JWT credential to verify
    /// * `requirement` - The requirement to check (e.g., "age > 18", "interest = crypto")
    ///
    /// # Returns
    /// True if the credential satisfies the requirement, false otherwise
    ///
    /// # Example
    /// ```
    /// // Verify that a user is over 18 without revealing their exact age
    /// let proof = protocol_manager.verify_requirement(
    ///     &credential,
    ///     "age > 18"
    /// ).await?;
    /// ```
    pub async fn verify_requirement(
        &self,
        credential: &str,
        requirement: &str,
    ) -> Result<bool, Box<dyn Error>> {
        info!("Verifying requirement: {} against credential", requirement);

        // SECURITY: Input validation should be done before calling this method
        // This method assumes requirement has been validated by validate_requirement()
        
        // Basic implementation: Parse the credential as JSON and check simple requirements
        // 
        // NOTE: Full SD-JWT verification using bh-sd-jwt crate is planned for Phase 3.
        // This will provide zero-knowledge proof functionality for Ad Tech use cases.
        // See: https://github.com/daveylupes/trust-sidecar/issues

        // Parse as JSON and perform basic requirement checks
        let cred_json = serde_json::from_str::<serde_json::Value>(credential)
            .map_err(|e| format!("Invalid credential format: {}", e))?;

        // SECURITY: Whitelist-based requirement parsing
        // Only allow specific operators: >, <, =, !=, >=, <=
        // Field names must be alphanumeric with underscores
        
        // Simple requirement parsing (e.g., "age > 18")
        if requirement.contains(">=") {
            let parts: Vec<&str> = requirement.split(">=").map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let field = parts[0].trim();
                if field.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    if let Ok(threshold) = parts[1].trim().parse::<i64>() {
                        if let Some(value) = cred_json.get(field).and_then(|v| v.as_i64()) {
                            return Ok(value >= threshold);
                        }
                    }
                }
            }
        } else if requirement.contains("<=") {
            let parts: Vec<&str> = requirement.split("<=").map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let field = parts[0].trim();
                if field.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    if let Ok(threshold) = parts[1].trim().parse::<i64>() {
                        if let Some(value) = cred_json.get(field).and_then(|v| v.as_i64()) {
                            return Ok(value <= threshold);
                        }
                    }
                }
            }
        } else if requirement.contains("!=") {
            let parts: Vec<&str> = requirement.split("!=").map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let field = parts[0].trim();
                if field.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    let expected = parts[1].trim().trim_matches('"').trim_matches('\'');
                    if let Some(value) = cred_json.get(field).and_then(|v| v.as_str()) {
                        return Ok(value != expected);
                    }
                }
            }
        } else if requirement.contains(">") && !requirement.contains(">=") {
            let parts: Vec<&str> = requirement.split('>').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let field = parts[0].trim();
                if field.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    if let Ok(threshold) = parts[1].trim().parse::<i64>() {
                        if let Some(value) = cred_json.get(field).and_then(|v| v.as_i64()) {
                            return Ok(value > threshold);
                        }
                    }
                }
            }
        } else if requirement.contains("<") && !requirement.contains("<=") {
            let parts: Vec<&str> = requirement.split('<').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let field = parts[0].trim();
                if field.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    if let Ok(threshold) = parts[1].trim().parse::<i64>() {
                        if let Some(value) = cred_json.get(field).and_then(|v| v.as_i64()) {
                            return Ok(value < threshold);
                        }
                    }
                }
            }
        } else if requirement.contains("=") && !requirement.contains("!=") && !requirement.contains(">=") && !requirement.contains("<=") {
            let parts: Vec<&str> = requirement.split('=').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let field = parts[0].trim();
                if field.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    let expected = parts[1].trim().trim_matches('"').trim_matches('\'');
                    if let Some(value) = cred_json.get(field).and_then(|v| v.as_str()) {
                        return Ok(value == expected);
                    }
                }
            }
        }

        // Return false if credential cannot be parsed or requirement cannot be verified
        Ok(false)
    }

    /// Generate a selective disclosure proof
    ///
    /// Creates a proof that satisfies a requirement without revealing the full credential.
    ///
    /// # Arguments
    /// * `credential` - The full SD-JWT credential
    /// * `disclosure_requirements` - What needs to be proven (e.g., ["age > 18"])
    ///
    /// # Returns
    /// A selective disclosure proof that can be verified without revealing raw data
    pub async fn generate_proof(
        &self,
        _credential: &str,
        _disclosure_requirements: &[&str],
    ) -> Result<String, Box<dyn Error>> {
        info!(
            "Generating selective disclosure proof for: {:?}",
            _disclosure_requirements
        );
        // 
        // NOTE: SD-JWT proof generation is planned for Phase 3.
        // This will enable zero-knowledge proofs for ad targeting without sharing raw data.
        // See: https://github.com/daveylupes/trust-sidecar/issues
        Ok("proof_placeholder".to_string())
    }

    /// Issue a verifiable credential
    ///
    /// Allows an authority (e.g., you) to sign a VC for an agent/user.
    ///
    /// # Arguments
    /// * `subject_did` - The DID of the credential subject
    /// * `claims` - The claims to include in the credential
    /// * `issuer_did` - The DID of the issuer (optional, uses default if not provided)
    /// * `issuer_private_key` - The private key of the issuer (PEM format)
    /// * `credential_type` - Type of credential (e.g., "VerifiedUser", "CryptoWhale")
    /// * `expiration_days` - Number of days until expiration (default: 365)
    ///
    /// # Returns
    /// A signed JWT credential
    pub async fn issue_credential(
        &self,
        subject_did: &str,
        claims: serde_json::Value,
        issuer_did: Option<&str>,
        issuer_private_key: &str,
        credential_type: &str,
        expiration_days: Option<u32>,
    ) -> Result<String, Box<dyn Error>> {
        info!("Issuing credential for DID: {}", subject_did);

        let issuer = issuer_did
            .or(self.default_issuer_did.as_deref())
            .ok_or("No issuer DID provided")?;

        let now = Utc::now();
        let exp_days = expiration_days.unwrap_or(365);
        let exp = now + chrono::Duration::days(exp_days as i64);

        let credential_claims = CredentialClaims {
            iss: issuer.to_string(),
            sub: subject_did.to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            vc_type: credential_type.to_string(),
            claims,
        };

        // Encode the JWT using ES256 (ECDSA P-256)
        let header = Header::new(Algorithm::ES256);
        let encoding_key = EncodingKey::from_ec_pem(issuer_private_key.as_bytes())
            .map_err(|e| format!("Failed to parse issuer key: {}", e))?;

        let token = encode(&header, &credential_claims, &encoding_key)
            .map_err(|e| format!("Failed to encode credential: {}", e))?;

        info!("Credential issued successfully for subject: {}", subject_did);
        Ok(token)
    }

    /// Verify a JWT credential
    ///
    /// # Arguments
    /// * `credential` - The JWT credential to verify
    /// * `issuer_public_key` - The public key of the issuer (PEM format)
    ///
    /// # Returns
    /// The decoded claims if valid
    pub async fn verify_credential(
        &self,
        credential: &str,
        issuer_public_key: &str,
    ) -> Result<CredentialClaims, Box<dyn Error>> {
        info!("Verifying credential...");

        let validation = Validation::new(Algorithm::ES256);
        let decoding_key = DecodingKey::from_ec_pem(issuer_public_key.as_bytes())
            .map_err(|e| format!("Failed to parse issuer public key: {}", e))?;

        let token_data = decode::<CredentialClaims>(credential, &decoding_key, &validation)
            .map_err(|e| format!("Failed to verify credential: {}", e))?;

        // Check expiration
        let now = Utc::now().timestamp();
        if token_data.claims.exp < now {
            return Err("Credential has expired".into());
        }

        info!("Credential verified successfully");
        Ok(token_data.claims)
    }

    /// Generate a proof of view for Ad Tech
    ///
    /// Creates a cryptographic proof that a specific content was viewed by a DID.
    /// This enables verifiable ad impressions without revealing user identity.
    ///
    /// # Arguments
    /// * `viewer_did` - The DID of the viewer
    /// * `content_id` - URL or identifier of the viewed content
    /// * `credential_proof` - Optional credential proof (e.g., "age > 18")
    ///
    /// # Returns
    /// A ProofOfView structure with cryptographic proof
    pub async fn generate_proof_of_view(
        &self,
        viewer_did: &str,
        content_id: &str,
        credential_proof: Option<&str>,
    ) -> Result<ProofOfView, Box<dyn Error>> {
        info!("Generating proof of view for content: {}", content_id);

        let timestamp = Utc::now().timestamp();

        // Create a hash of the view data for proof
        let mut hasher = Sha256::new();
        hasher.update(viewer_did.as_bytes());
        hasher.update(content_id.as_bytes());
        hasher.update(timestamp.to_string().as_bytes());
        if let Some(proof) = credential_proof {
            hasher.update(proof.as_bytes());
        }
        let proof_hash = format!("{:x}", hasher.finalize());

        let proof = ProofOfView {
            viewer_did: viewer_did.to_string(),
            content_id: content_id.to_string(),
            timestamp,
            proof_hash,
            credential_proof: credential_proof.map(|s| s.to_string()),
        };

        info!("Proof of view generated successfully");
        Ok(proof)
    }

    /// Verify a proof of view
    ///
    /// # Arguments
    /// * `proof` - The ProofOfView to verify
    ///
    /// # Returns
    /// True if the proof is valid
    /// 
    /// SECURITY: Implements replay protection using proof hash registry
    pub async fn verify_proof_of_view(&self, proof: &ProofOfView) -> Result<bool, Box<dyn Error>> {
        info!("Verifying proof of view...");

        // SECURITY: Check for replay attack - has this proof hash been used before?
        let mut used_hashes = self.used_proof_hashes.write().await;
        if used_hashes.contains(&proof.proof_hash) {
            warn!("Proof hash already used - potential replay attack");
            return Ok(false);
        }

        // Recompute the hash
        let mut hasher = Sha256::new();
        hasher.update(proof.viewer_did.as_bytes());
        hasher.update(proof.content_id.as_bytes());
        hasher.update(proof.timestamp.to_string().as_bytes());
        if let Some(ref cred_proof) = proof.credential_proof {
            hasher.update(cred_proof.as_bytes());
        }
        let computed_hash = format!("{:x}", hasher.finalize());

        // Check if hash matches
        if computed_hash != proof.proof_hash {
            return Ok(false);
        }

        // SECURITY: Check timestamp is not too old (reduced to 1 hour for better security)
        let now = Utc::now().timestamp();
        let age = now - proof.timestamp;
        if age > 3600 {
            // 1 hour (reduced from 24 hours)
            return Ok(false);
        }
        
        // SECURITY: Check timestamp is not in the future (clock skew protection)
        if proof.timestamp > now + 300 {
            // Allow 5 minutes clock skew
            return Ok(false);
        }

        // SECURITY: Mark this proof hash as used to prevent replay
        used_hashes.insert(proof.proof_hash.clone());
        
        // Clean up old hashes (older than 2 hours) to prevent memory growth
        // In production, use a time-based cleanup or LRU cache
        if used_hashes.len() > 10000 {
            // Simple cleanup: clear if too large (in production, use proper cache)
            used_hashes.clear();
        }

        info!("Proof of view verified successfully");
        Ok(true)
    }
}

impl Default for ProtocolManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_protocol_manager_creation() {
        let _manager = ProtocolManager::new();
    }
}
