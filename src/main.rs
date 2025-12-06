//! Trust Sidecar - A universal identity and messaging layer for AI Agents & Ad Tech
//!
//! Provides Identity (DID), Encryption (DIDComm), and Verifiable Credentials (SD-JWT)
//! to any application via a lightweight Rust sidecar.

mod cli;
mod identity;
mod messaging;
mod protocols;

use affinidi_messaging_sdk::config::ATMConfig;
use affinidi_tdk_common::TDKSharedState;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use cli::{Cli, Commands};
use identity::IdentityManager;
use messaging::MessagingManager;
use protocols::ProtocolManager;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::sync::Arc;
use tracing::{error, info};

/// Application state shared across API handlers
#[derive(Clone)]
struct AppState {
    identity: Arc<IdentityManager>,
    messaging: Arc<MessagingManager>,
    protocols: Arc<ProtocolManager>,
}

/// Health check endpoint
async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "trust-sidecar",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// API request to generate a new DID
#[derive(Deserialize)]
struct GenerateDidRequest {
    service_id: Option<String>,
}

/// API response with a DID
#[derive(Serialize)]
struct DidResponse {
    did: String,
}

/// Generate a new DID endpoint
async fn generate_did(
    State(state): State<AppState>,
    Json(req): Json<GenerateDidRequest>,
) -> Result<Json<DidResponse>, StatusCode> {
    match state.identity.generate_did(req.service_id.as_deref()).await {
        Ok(did) => Ok(Json(DidResponse { did })),
        Err(e) => {
            error!("Failed to generate DID: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// API request to verify a credential requirement
#[derive(Deserialize)]
struct VerifyRequirementRequest {
    credential: String,
    requirement: String,
}

/// API response with verification result
#[derive(Serialize)]
struct VerifyRequirementResponse {
    verified: bool,
}

/// Verify credential requirement endpoint
async fn verify_requirement(
    State(state): State<AppState>,
    Json(req): Json<VerifyRequirementRequest>,
) -> Result<Json<VerifyRequirementResponse>, StatusCode> {
    match state
        .protocols
        .verify_requirement(&req.credential, &req.requirement)
        .await
    {
        Ok(verified) => Ok(Json(VerifyRequirementResponse { verified })),
        Err(e) => {
            error!("Failed to verify requirement: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn run_server(host: String, port: u16) -> Result<(), Box<dyn Error>> {
    info!("Starting Trust Sidecar v{}...", env!("CARGO_PKG_VERSION"));

    info!("Initializing TDK Shared State...");
    let tdk_shared_state = TDKSharedState::default().await;

    info!("Initializing Identity Manager...");
    let identity_manager = Arc::new(IdentityManager::new(tdk_shared_state.clone()));

    info!("Initializing Messaging Manager...");
    let config = ATMConfig::builder().build()?;
    let messaging_manager = Arc::new(
        MessagingManager::new(config, tdk_shared_state)
            .await
            .map_err(|e| format!("Failed to initialize messaging: {}", e))?,
    );

    info!("Initializing Protocol Manager...");
    let protocol_manager = Arc::new(ProtocolManager::new());

    // Set up API server
    let app_state = AppState {
        identity: identity_manager,
        messaging: messaging_manager,
        protocols: protocol_manager,
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/did/generate", post(generate_did))
        .route("/api/v1/protocols/verify", post(verify_requirement))
        .with_state(app_state);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

    info!("Trust Sidecar API server listening on http://{}", addr);
    info!("Endpoints:");
    info!("  GET  /health - Health check");
    info!("  POST /api/v1/did/generate - Generate a new DID");
    info!("  POST /api/v1/protocols/verify - Verify a credential requirement");
    info!("Waiting for requests... (Ctrl+C to exit)");

    // Start the server
    axum::serve(listener, app)
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Serve { port, host }) => run_server(host, port).await,
        Some(Commands::GenerateDid { service_id }) => {
            info!("Generating DID...");
            let tdk = TDKSharedState::default().await;
            let identity = IdentityManager::new(tdk);
            let did = identity.generate_did(service_id.as_deref()).await?;
            println!("Generated DID: {}", did);
            Ok(())
        }
        Some(Commands::Verify {
            credential,
            requirement,
        }) => {
            info!("Verifying credential requirement...");

            // Attempt to read from file, otherwise treat as JSON string
            let cred_str = if fs::metadata(&credential).is_ok() {
                fs::read_to_string(&credential)?
            } else {
                credential
            };

            let protocol = ProtocolManager::new();
            let verified = protocol.verify_requirement(&cred_str, &requirement).await?;

            if verified {
                println!("Requirement '{}' verified successfully", requirement);
            } else {
                println!("Requirement '{}' verification failed", requirement);
            }

            Ok(())
        }
        None => run_server("127.0.0.1".to_string(), 3000).await,
    }
}
