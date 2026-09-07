//! Agent spend-authorization credentials
//!
//! Lets a principal (a human/user DID) grant an agent (an agent DID) a
//! constrained, standing authorization to spend on their behalf, and lets
//! anyone check a proposed transaction against that authorization.
//!
//! `max_transaction_amount` is a per-transaction cap on a reusable
//! authorization (like a card with a per-transaction limit), not a
//! single-use grant — the same signed credential can be checked against many
//! transactions, each evaluated independently. There is no cumulative
//! running total and no revocation registry; both are out of scope for this
//! phase.
//!
//! Issuance requires the principal's private key and is CLI-only (see
//! `cli.rs`), following the same precedent as `issue_credential`: private
//! key material is never accepted over HTTP. Verification only needs the
//! issuer's public key and is safe to expose over HTTP.

use super::ProtocolManager;
use super::TypedCredential;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Claims carried inside a SpendAuthorization credential's JWT body.
///
/// Deliberately does not repeat `principal_did`/`agent_did` here — those are
/// already carried by the wrapping JWT's `iss` (principal) / `sub` (agent),
/// available via `TypedCredential::issuer_did` / `subject_did` after
/// verification. Duplicating them in the claims body would create a
/// spoofing surface if the two ever disagreed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendAuthorizationClaims {
    /// Per-transaction cap on a standing authorization
    pub max_transaction_amount: f64,
    /// ISO 4217 currency code, e.g. "USD"
    pub currency: String,
    /// If present, only these merchants are authorized
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merchant_allowlist: Option<Vec<String>>,
    /// If present, only these categories are authorized
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category_allowlist: Option<Vec<String>>,
    /// Business-authorization window start (unix seconds). Independent of
    /// the JWT's own `iat`/`exp` — this governs when the *authorization*
    /// is active, not when the *JWT* is technically valid.
    pub valid_from: i64,
    /// Business-authorization window end (unix seconds)
    pub valid_until: i64,
    /// Grant identifier (not single-use) — useful as an `authorization_ref`
    /// linking a later `ProofOfAction` back to this grant
    pub nonce: String,
}

/// A transaction an agent proposes to make, to be checked against a
/// `SpendAuthorizationClaims`.
#[derive(Debug, Clone, Deserialize)]
pub struct ProposedTransaction {
    pub amount: f64,
    pub currency: String,
    pub merchant_id: String,
    #[serde(default)]
    pub category: Option<String>,
    pub timestamp: i64,
}

/// Result of checking a `ProposedTransaction` against a `SpendAuthorizationClaims`.
#[derive(Debug, Clone, Serialize)]
pub struct SpendAuthorizationCheckResult {
    pub authorized: bool,
    /// Machine-readable reason codes; empty iff `authorized` is true.
    pub reasons: Vec<String>,
}

/// Pure, synchronous constraint check — no I/O, trivially unit-testable.
pub fn check_transaction_against_authorization(
    claims: &SpendAuthorizationClaims,
    tx: &ProposedTransaction,
) -> SpendAuthorizationCheckResult {
    let mut reasons = Vec::new();

    if tx.amount <= 0.0 || tx.amount > claims.max_transaction_amount {
        reasons.push("amount_exceeds_max".to_string());
    }
    if !tx.currency.eq_ignore_ascii_case(&claims.currency) {
        reasons.push("currency_mismatch".to_string());
    }
    if tx.timestamp < claims.valid_from || tx.timestamp > claims.valid_until {
        reasons.push("outside_valid_window".to_string());
    }
    if let Some(ref allowlist) = claims.merchant_allowlist
        && !allowlist.iter().any(|m| m == &tx.merchant_id)
    {
        reasons.push("merchant_not_allowed".to_string());
    }
    if let Some(ref allowlist) = claims.category_allowlist {
        match &tx.category {
            Some(cat) if allowlist.iter().any(|c| c == cat) => {}
            _ => reasons.push("category_not_allowed".to_string()),
        }
    }

    let authorized = reasons.is_empty();
    SpendAuthorizationCheckResult { authorized, reasons }
}

impl ProtocolManager {
    /// Issue a SpendAuthorization credential, signed by the principal.
    /// CLI-only in practice: `principal_private_key` must never be accepted
    /// from an HTTP request.
    pub async fn issue_spend_authorization(
        &self,
        principal_did: &str,
        principal_private_key: &str,
        agent_did: &str,
        claims: SpendAuthorizationClaims,
        expiration_days: Option<u32>,
    ) -> Result<String, Box<dyn Error>> {
        self.issue_typed_credential(
            agent_did,
            &claims,
            principal_did,
            principal_private_key,
            "SpendAuthorization",
            expiration_days,
        )
        .await
    }

    /// Verify a SpendAuthorization credential's signature/expiry and decode
    /// its claims. Only needs the principal's public key.
    pub async fn verify_spend_authorization(
        &self,
        credential: &str,
        issuer_public_key: &str,
    ) -> Result<TypedCredential<SpendAuthorizationClaims>, Box<dyn Error>> {
        self.verify_typed_credential(credential, issuer_public_key, "SpendAuthorization")
            .await
    }

    /// Verify a SpendAuthorization credential and check a proposed
    /// transaction against its constraints in one call. Safe to expose over
    /// HTTP: takes only a signed credential, the issuer's public key, and
    /// the proposed transaction — never private key material.
    pub async fn verify_transaction_authorization(
        &self,
        credential: &str,
        issuer_public_key: &str,
        tx: &ProposedTransaction,
    ) -> Result<
        (
            TypedCredential<SpendAuthorizationClaims>,
            SpendAuthorizationCheckResult,
        ),
        Box<dyn Error>,
    > {
        let typed = self.verify_spend_authorization(credential, issuer_public_key).await?;
        let result = check_transaction_against_authorization(&typed.claims, tx);
        Ok((typed, result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_claims() -> SpendAuthorizationClaims {
        SpendAuthorizationClaims {
            max_transaction_amount: 50.0,
            currency: "USD".to_string(),
            merchant_allowlist: Some(vec!["merchant-123".to_string()]),
            category_allowlist: Some(vec!["groceries".to_string()]),
            valid_from: 1_000,
            valid_until: 2_000,
            nonce: "test-nonce".to_string(),
        }
    }

    fn base_tx() -> ProposedTransaction {
        ProposedTransaction {
            amount: 25.0,
            currency: "USD".to_string(),
            merchant_id: "merchant-123".to_string(),
            category: Some("groceries".to_string()),
            timestamp: 1_500,
        }
    }

    #[test]
    fn test_authorized_when_all_constraints_satisfied() {
        let result = check_transaction_against_authorization(&base_claims(), &base_tx());
        assert!(result.authorized);
        assert!(result.reasons.is_empty());
    }

    #[test]
    fn test_amount_exceeds_max() {
        let mut tx = base_tx();
        tx.amount = 999.0;
        let result = check_transaction_against_authorization(&base_claims(), &tx);
        assert!(!result.authorized);
        assert!(result.reasons.contains(&"amount_exceeds_max".to_string()));
    }

    #[test]
    fn test_zero_or_negative_amount_rejected() {
        let mut tx = base_tx();
        tx.amount = 0.0;
        let result = check_transaction_against_authorization(&base_claims(), &tx);
        assert!(result.reasons.contains(&"amount_exceeds_max".to_string()));
    }

    #[test]
    fn test_currency_mismatch() {
        let mut tx = base_tx();
        tx.currency = "EUR".to_string();
        let result = check_transaction_against_authorization(&base_claims(), &tx);
        assert!(result.reasons.contains(&"currency_mismatch".to_string()));
    }

    #[test]
    fn test_outside_valid_window_before() {
        let mut tx = base_tx();
        tx.timestamp = 500;
        let result = check_transaction_against_authorization(&base_claims(), &tx);
        assert!(result.reasons.contains(&"outside_valid_window".to_string()));
    }

    #[test]
    fn test_outside_valid_window_after() {
        let mut tx = base_tx();
        tx.timestamp = 2_500;
        let result = check_transaction_against_authorization(&base_claims(), &tx);
        assert!(result.reasons.contains(&"outside_valid_window".to_string()));
    }

    #[test]
    fn test_merchant_not_allowed() {
        let mut tx = base_tx();
        tx.merchant_id = "merchant-999".to_string();
        let result = check_transaction_against_authorization(&base_claims(), &tx);
        assert!(result.reasons.contains(&"merchant_not_allowed".to_string()));
    }

    #[test]
    fn test_category_not_allowed() {
        let mut tx = base_tx();
        tx.category = Some("electronics".to_string());
        let result = check_transaction_against_authorization(&base_claims(), &tx);
        assert!(result.reasons.contains(&"category_not_allowed".to_string()));
    }

    #[test]
    fn test_category_missing_when_required() {
        let mut tx = base_tx();
        tx.category = None;
        let result = check_transaction_against_authorization(&base_claims(), &tx);
        assert!(result.reasons.contains(&"category_not_allowed".to_string()));
    }

    #[test]
    fn test_no_allowlists_means_unrestricted() {
        let mut claims = base_claims();
        claims.merchant_allowlist = None;
        claims.category_allowlist = None;
        let mut tx = base_tx();
        tx.merchant_id = "any-merchant".to_string();
        tx.category = None;
        let result = check_transaction_against_authorization(&claims, &tx);
        assert!(result.authorized);
    }

    #[test]
    fn test_multiple_violations_all_reported() {
        let mut tx = base_tx();
        tx.amount = 999.0;
        tx.currency = "EUR".to_string();
        let result = check_transaction_against_authorization(&base_claims(), &tx);
        assert!(!result.authorized);
        assert!(result.reasons.len() >= 2);
    }

    #[tokio::test]
    async fn test_issue_and_verify_spend_authorization_round_trip() -> Result<(), Box<dyn Error>> {
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
        let claims = base_claims();

        let jwt = manager
            .issue_spend_authorization(
                "did:key:zPrincipal",
                private_key,
                "did:key:zAgent",
                claims,
                Some(30),
            )
            .await?;

        let (typed, result) = manager
            .verify_transaction_authorization(&jwt, public_key, &base_tx())
            .await?;

        assert_eq!(typed.issuer_did, "did:key:zPrincipal");
        assert_eq!(typed.subject_did, "did:key:zAgent");
        assert!(result.authorized);

        // Same credential, a second, still-within-limits transaction: not single-use.
        let mut second_tx = base_tx();
        second_tx.amount = 10.0;
        let (_, second_result) = manager
            .verify_transaction_authorization(&jwt, public_key, &second_tx)
            .await?;
        assert!(second_result.authorized);

        Ok(())
    }
}
