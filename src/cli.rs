//! CLI Module
//!
//! Command-line interface for the Trust Sidecar

use clap::{Parser, Subcommand};

/// Trust Sidecar CLI - A universal identity and messaging layer
#[derive(Parser)]
#[command(name = "trust-sidecar")]
#[command(about = "Trust Sidecar - Identity and messaging layer for AI Agents & Ad Tech", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the Trust Sidecar API server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },

    /// Generate a new DID
    GenerateDid {
        /// Service ID to associate with this DID
        #[arg(short, long)]
        service_id: Option<String>,
    },

    /// Verify a credential requirement
    Verify {
        /// The credential to verify (JSON string or file path)
        #[arg(short, long)]
        credential: String,

        /// The requirement to check (e.g., "age > 18")
        #[arg(short, long)]
        requirement: String,
    },

    /// Issue a verifiable credential
    IssueCredential {
        /// The DID of the subject
        #[arg(long)]
        subject_did: String,

        /// The claims as JSON string or file path
        #[arg(long)]
        claims: String,

        /// The DID of the issuer
        #[arg(long)]
        issuer_did: String,

        /// Path to issuer private key (PEM format)
        #[arg(long)]
        issuer_key: String,

        /// Type of credential (e.g., "VerifiedUser")
        #[arg(long)]
        credential_type: String,

        /// Expiration in days (default: 365)
        #[arg(long, default_value = "365")]
        expiration_days: u32,
    },

    /// Generate a proof of view (for Ad Tech)
    GenerateProofOfView {
        /// The DID of the viewer
        #[arg(long)]
        viewer_did: String,

        /// Content ID or URL
        #[arg(long)]
        content_id: String,

        /// Optional credential proof requirement
        #[arg(long)]
        credential_proof: Option<String>,
    },

    /// Issue a spend authorization credential (principal -> agent).
    /// CLI-only: the principal's private key never leaves this machine.
    IssueSpendAuthorization {
        /// DID of the principal (human/user) granting authorization
        #[arg(long)]
        principal_did: String,

        /// Path to the principal's private key (PEM format)
        #[arg(long)]
        principal_key: String,

        /// DID of the agent being authorized
        #[arg(long)]
        agent_did: String,

        /// Maximum amount authorized per transaction
        #[arg(long)]
        max_amount: f64,

        /// ISO 4217 currency code
        #[arg(long, default_value = "USD")]
        currency: String,

        /// Comma-separated list of allowed merchant IDs (omit for no restriction)
        #[arg(long)]
        merchants: Option<String>,

        /// Comma-separated list of allowed categories (omit for no restriction)
        #[arg(long)]
        categories: Option<String>,

        /// Unix timestamp the authorization becomes active (default: now)
        #[arg(long)]
        valid_from: Option<i64>,

        /// Length of the authorization window in days, used if --valid-until is not given
        #[arg(long, default_value = "30")]
        valid_days: i64,

        /// Unix timestamp the authorization expires (overrides --valid-days)
        #[arg(long)]
        valid_until: Option<i64>,

        /// Technical JWT expiration in days (default: 365)
        #[arg(long, default_value = "365")]
        expiration_days: u32,
    },

    /// Verify a spend authorization credential against a proposed transaction
    VerifySpendAuthorization {
        /// The credential to verify (JWT string or file path)
        #[arg(long)]
        credential: String,

        /// Path to the issuer's (principal's) public key (PEM format)
        #[arg(long)]
        issuer_public_key: String,

        /// Proposed transaction amount
        #[arg(long)]
        amount: f64,

        /// Proposed transaction currency
        #[arg(long)]
        currency: String,

        /// Proposed transaction merchant ID
        #[arg(long)]
        merchant_id: String,

        /// Proposed transaction category
        #[arg(long)]
        category: Option<String>,

        /// Proposed transaction timestamp (unix seconds, default: now)
        #[arg(long)]
        timestamp: Option<i64>,
    },
}

impl Cli {
    /// Parse command line arguments
    pub fn parse() -> Self {
        Parser::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: deliberately does not call `Cli::parse()` (which parses the real
    // process argv via `std::env::args()`). Under `cargo test`, that argv can
    // contain harness flags like `--quiet` or `--test-threads`, which clap
    // rejects with a hard `std::process::exit()` — aborting the entire test
    // binary mid-run, not just this test. `try_parse_from` takes a fixed,
    // known argument vector instead.

    #[test]
    fn test_cli_parse_generate_did() {
        let cli = Cli::try_parse_from(["trust-sidecar", "generate-did", "--service-id", "test"])
            .expect("should parse a valid generate-did invocation");
        assert!(matches!(cli.command, Some(Commands::GenerateDid { .. })));
    }

    #[test]
    fn test_cli_parse_no_subcommand() {
        let cli = Cli::try_parse_from(["trust-sidecar"]).expect("no subcommand should be valid");
        assert!(cli.command.is_none());
    }

    /// Regression test: `IssueCredential`'s fields used to derive colliding
    /// short flags (`claims`/`credential_type` both `-c`, `issuer_did`/
    /// `issuer_key` both `-i`), which made clap panic (a debug assertion) any
    /// time the CLI's command tree was built at all — i.e. every invocation
    /// of the binary, not just `issue-credential` itself.
    #[test]
    fn test_cli_parse_issue_credential() {
        let cli = Cli::try_parse_from([
            "trust-sidecar",
            "issue-credential",
            "--subject-did",
            "did:key:zSubject",
            "--claims",
            "{}",
            "--issuer-did",
            "did:key:zIssuer",
            "--issuer-key",
            "/tmp/key.pem",
            "--credential-type",
            "VerifiedUser",
        ])
        .expect("should parse a valid issue-credential invocation");
        assert!(matches!(cli.command, Some(Commands::IssueCredential { .. })));
    }

    #[test]
    fn test_cli_parse_issue_spend_authorization() {
        let cli = Cli::try_parse_from([
            "trust-sidecar",
            "issue-spend-authorization",
            "--principal-did",
            "did:key:zPrincipal",
            "--principal-key",
            "/tmp/principal.pem",
            "--agent-did",
            "did:key:zAgent",
            "--max-amount",
            "50.0",
        ])
        .expect("should parse a valid issue-spend-authorization invocation");
        assert!(matches!(
            cli.command,
            Some(Commands::IssueSpendAuthorization { .. })
        ));
    }
}
