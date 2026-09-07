//! Typed credential helpers
//!
//! Thin, generic wrapper around `ProtocolManager::issue_credential` /
//! `verify_credential`. Deliberately does not duplicate any JWT signing or
//! verification logic — every credential "type" added on top of this (see
//! `agent_auth.rs`) is just a plain serde struct plumbed through here, so
//! there is exactly one ES256 sign/verify code path in the crate.

use super::ProtocolManager;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;

/// A verified credential with its claims deserialized into a typed struct,
/// instead of the raw, crate-private `CredentialClaims`/`serde_json::Value`.
#[derive(Debug, Clone, Serialize)]
pub struct TypedCredential<T> {
    pub issuer_did: String,
    pub subject_did: String,
    pub issued_at: i64,
    pub expires_at: i64,
    pub credential_type: String,
    pub claims: T,
}

impl ProtocolManager {
    /// Issue a credential whose claims are a typed struct rather than a raw
    /// `serde_json::Value`. Serializes `claims` and delegates to the existing
    /// `issue_credential` for the actual ES256 signing.
    pub async fn issue_typed_credential<T: Serialize>(
        &self,
        subject_did: &str,
        claims: &T,
        issuer_did: &str,
        issuer_private_key: &str,
        credential_type: &str,
        expiration_days: Option<u32>,
    ) -> Result<String, Box<dyn Error>> {
        let value = serde_json::to_value(claims)
            .map_err(|e| format!("Failed to serialize claims: {}", e))?;
        self.issue_credential(
            subject_did,
            value,
            Some(issuer_did),
            issuer_private_key,
            credential_type,
            expiration_days,
        )
        .await
    }

    /// Verify a credential and deserialize its claims into `T`, rejecting it
    /// if the credential's `vc_type` doesn't match `expected_type` or if the
    /// claims don't match `T`'s shape.
    pub async fn verify_typed_credential<T: DeserializeOwned>(
        &self,
        credential: &str,
        issuer_public_key: &str,
        expected_type: &str,
    ) -> Result<TypedCredential<T>, Box<dyn Error>> {
        let raw = self.verify_credential(credential, issuer_public_key).await?;

        if raw.vc_type != expected_type {
            return Err(format!(
                "Unexpected credential type: expected '{}', got '{}'",
                expected_type, raw.vc_type
            )
            .into());
        }

        let claims: T = serde_json::from_value(raw.claims)
            .map_err(|e| format!("Claims do not match expected schema: {}", e))?;

        Ok(TypedCredential {
            issuer_did: raw.iss,
            subject_did: raw.sub,
            issued_at: raw.iat,
            expires_at: raw.exp,
            credential_type: raw.vc_type,
            claims,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestClaims {
        foo: String,
        n: i64,
    }

    #[tokio::test]
    async fn test_typed_credential_round_trip() -> Result<(), Box<dyn Error>> {
        // ES256 (P-256) key pair generated once for this test via:
        //   openssl ecparam -genkey -name prime256v1 -noout -out priv.pem
        //   openssl pkcs8 -topk8 -nocrypt -in priv.pem -out priv_pkcs8.pem
        //   openssl ec -in priv.pem -pubout -out pub.pem
        let private_key = "-----BEGIN PRIVATE KEY-----\n\
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQg256cNmZQKmuT5+tQ\n\
mRBdyNkOM4NV9dmB0r7L3PXdc+ahRANCAARpewC22e3g15sEooO3ilfwq8GebooE\n\
O3JmPyzpSj0N+3WHZlESaUVqIPTfMqArb//9hdFBjFIfF49F5JrrsTQo\n\
-----END PRIVATE KEY-----\n";
        let public_key = "-----BEGIN PUBLIC KEY-----\n\
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEaXsAttnt4NebBKKDt4pX8KvBnm6K\n\
BDtyZj8s6Uo9Dft1h2ZREmlFaiD03zKgK2///YXRQYxSHxePReSa67E0KA==\n\
-----END PUBLIC KEY-----\n";

        let manager = ProtocolManager::new();
        let claims = TestClaims { foo: "bar".to_string(), n: 42 };

        let jwt = manager
            .issue_typed_credential(
                "did:key:zSubject",
                &claims,
                "did:key:zIssuer",
                private_key,
                "TestType",
                Some(30),
            )
            .await?;

        let verified: TypedCredential<TestClaims> = manager
            .verify_typed_credential(&jwt, public_key, "TestType")
            .await?;

        assert_eq!(verified.claims, claims);
        assert_eq!(verified.issuer_did, "did:key:zIssuer");
        assert_eq!(verified.subject_did, "did:key:zSubject");
        assert_eq!(verified.credential_type, "TestType");

        // Wrong expected type should be rejected
        let wrong_type = manager
            .verify_typed_credential::<TestClaims>(&jwt, public_key, "OtherType")
            .await;
        assert!(wrong_type.is_err());

        Ok(())
    }
}
