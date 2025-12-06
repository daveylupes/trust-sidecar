//! Messaging Module
//!
//! Handles secure DIDComm messaging between agents/users.
//! Wraps the Affinidi Messaging SDK (ATM) for encrypted peer-to-peer communication.

use affinidi_messaging_sdk::{ATM, config::ATMConfig};
use affinidi_tdk_common::TDKSharedState;
use std::error::Error;
use tracing::info;

/// Messaging manager for secure DIDComm communication
///
/// Provides encrypted, peer-to-peer messaging capabilities using DIDComm protocol.
pub struct MessagingManager {
    atm: ATM,
}

impl MessagingManager {
    /// Create a new MessagingManager
    ///
    /// Initializes the Affinidi Trust Messaging (ATM) engine with the provided configuration.
    pub async fn new(
        config: ATMConfig,
        tdk_shared_state: TDKSharedState,
    ) -> Result<Self, Box<dyn Error>> {
        info!("Initializing MessagingManager...");
        let atm = ATM::new(config, tdk_shared_state)
            .await
            .map_err(|e| format!("Failed to initialize ATM: {:?}", e))?;

        info!("MessagingManager initialized successfully");
        Ok(Self { atm })
    }

    /// Send an encrypted message to a recipient DID
    ///
    /// # Arguments
    /// * `recipient_did` - The DID of the message recipient
    /// * `message` - The message payload (will be encrypted)
    ///
    /// # Returns
    /// The message ID if successful
    pub async fn send_message(
        &self,
        recipient_did: &str,
        _message: serde_json::Value,
    ) -> Result<String, Box<dyn Error>> {
        info!("Sending message to: {}", recipient_did);
        // TODO: Implement message sending using ATM profiles
        // NOTE: This is a placeholder implementation. Full DIDComm messaging is planned.
        Err("DIDComm message sending not yet implemented. Requires ATM profile setup.".into())
    }

    /// Receive and decrypt messages
    ///
    /// Returns a stream of incoming messages
    pub async fn receive_messages(&self) -> Result<Vec<serde_json::Value>, Box<dyn Error>> {
        info!("Checking for incoming messages...");
        // TODO: Implement message receiving using ATM profiles
        // NOTE: This is a placeholder implementation. Full DIDComm messaging is planned.
        Err("DIDComm message receiving not yet implemented. Requires ATM profile setup.".into())
    }

    /// Get the underlying ATM instance (for advanced usage)
    pub fn get_atm(&self) -> &ATM {
        &self.atm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_messaging_manager_creation() {
        let config = ATMConfig::builder().build().unwrap();
        let tdk = TDKSharedState::default().await;
        let manager = MessagingManager::new(config, tdk).await;
        assert!(manager.is_ok());
    }
}
