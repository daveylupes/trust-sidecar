//! Identity Module
//!
//! Handles DID (Decentralized Identifier) generation, key management, and credential storage.
//! Uses the OS keychain for secure private key storage.

use affinidi_did_key::DIDKey;
use affinidi_secrets_resolver::secrets::KeyType;
use affinidi_tdk_common::{TDKSharedState, secrets::save_secrets_locally};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Identity manager for the Trust Sidecar
///
/// Manages DIDs, private keys, and credentials for agents/users.
pub struct IdentityManager {
    tdk_shared_state: TDKSharedState,
    /// In-memory cache of DIDs by service_id
    /// In production, this would be persisted to keychain
    did_cache: Arc<RwLock<HashMap<String, String>>>,
    /// In-memory cache of credentials by DID
    /// In production, this would be persisted to keychain
    credential_cache: Arc<RwLock<HashMap<String, Vec<serde_json::Value>>>>,
}

impl IdentityManager {
    /// Create a new IdentityManager with the provided TDK shared state
    pub fn new(tdk_shared_state: TDKSharedState) -> Self {
        Self {
            tdk_shared_state,
            did_cache: Arc::new(RwLock::new(HashMap::new())),
            credential_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a new DID (Decentralized Identifier) for an agent/user
    ///
    /// # Arguments
    /// * `service_id` - Optional service identifier to associate with this DID
    ///
    /// # Returns
    /// The DID string (e.g., "did:key:z6Mk...")
    pub async fn generate_did(&self, service_id: Option<&str>) -> Result<String, Box<dyn Error>> {
        info!("Generating new DID...");

        // Generate a P256 key for optimal security and compatibility
        let (did, secret) = DIDKey::generate(KeyType::P256)
            .map_err(|e| format!("Failed to generate DID: {:?}", e))?;

        info!("Generated DID: {}", did);

        // Store the secret in the keychain using TDK
        if let Some(service_id) = service_id {
            save_secrets_locally(service_id, &did, &[secret])
                .map_err(|e| format!("Failed to save secrets: {:?}", e))?;

            // Load secrets into the resolver
            self.tdk_shared_state
                .load_secrets(service_id, &did)
                .await
                .map_err(|e| format!("Failed to load secrets: {:?}", e))?;

            // Cache the DID
            self.did_cache
                .write()
                .await
                .insert(service_id.to_string(), did.clone());
        }

        Ok(did)
    }

    /// Load an existing DID from the keychain
    pub async fn load_did(&self, service_id: &str) -> Result<Option<String>, Box<dyn Error>> {
        info!("Loading DID for service: {}", service_id);

        // Check cache first
        if let Some(did) = self.did_cache.read().await.get(service_id) {
            return Ok(Some(did.clone()));
        }

        // NOTE: Keychain loading using TDK is planned for future enhancement.
        // Currently returns None if not in cache. DIDs are loaded when generated.
        // See: https://github.com/daveylupes/trust-sidecar/issues
        Ok(None)
    }

    /// Store credentials for a DID in the keychain
    pub async fn store_credential(
        &self,
        did: &str,
        _credential: serde_json::Value,
    ) -> Result<(), Box<dyn Error>> {
        info!("Storing credential for DID: {}", did);

        // Store in cache
        let mut cache = self.credential_cache.write().await;
        cache
            .entry(did.to_string())
            .or_insert_with(Vec::new)
            .push(_credential);

        // NOTE: Keychain persistence using TDK is planned for future enhancement.
        // Currently credentials are stored in memory cache only.
        // See: https://github.com/daveylupes/trust-sidecar/issues
        Ok(())
    }

    /// Retrieve credentials for a DID
    pub async fn get_credentials(
        &self,
        did: &str,
    ) -> Result<Vec<serde_json::Value>, Box<dyn Error>> {
        info!("Retrieving credentials for DID: {}", did);

        // Get from cache
        let cache = self.credential_cache.read().await;
        Ok(cache.get(did).cloned().unwrap_or_default())
    }

    /// Get issuer's private key from keychain
    /// SECURITY: Loads key from OS keychain instead of accepting it via API
    pub async fn get_issuer_private_key(
        &self,
        service_id: &str,
        issuer_did: &str,
    ) -> Result<String, Box<dyn Error>> {
        info!("Loading issuer private key for service: {}, DID: {}", service_id, issuer_did);
        
        // Load DID first to ensure it exists
        let did = self.load_did(service_id).await?
            .ok_or_else(|| format!("DID not found for service_id: {}", service_id))?;
        
        if did != issuer_did {
            return Err(format!("DID mismatch: expected {}, got {}", issuer_did, did).into());
        }
        
        // NOTE: TDK keychain access for private key retrieval
        // The TDK resolver should have the key loaded from load_secrets
        // For now, we'll need to use the resolver's internal methods
        // This is a placeholder - actual implementation depends on TDK API
        
        // In a real implementation, we would:
        // 1. Use TDK resolver to get the key material
        // 2. Convert to PEM format
        // 3. Return the key
        
        // For now, return an error indicating this needs TDK integration
        Err("Private key retrieval from keychain requires TDK resolver integration. Use CLI with --issuer-key file for now.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_identity_manager_creation() {
        let tdk = TDKSharedState::default().await;
        let _manager = IdentityManager::new(tdk);
    }
}
