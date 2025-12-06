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

        // TODO: Load from keychain using TDK
        // Currently returns None if not in cache
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

        // TODO: Persist to keychain using TDK
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
