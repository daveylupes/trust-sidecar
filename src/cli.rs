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

    #[test]
    fn test_cli_parse() {
        let _cli = Cli::parse();
    }
}
