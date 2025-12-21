//! Trust Sidecar - A universal identity and messaging layer for AI Agents & Ad Tech
//!
//! Provides Identity (DID), Encryption (DIDComm), and Verifiable Credentials (SD-JWT)
//! to any application via a lightweight Rust sidecar.

mod cli;
mod identity;
mod messaging;
mod protocols;
mod security;

use affinidi_messaging_sdk::config::ATMConfig;
use affinidi_tdk_common::TDKSharedState;
use axum::{
    Router,
    extract::State,
    http::{StatusCode, header},
    response::Json,
    routing::{get, post},
};
use tower::ServiceBuilder;
use tower_http::{
    cors::{CorsLayer, AllowOrigin},
    limit::RequestBodyLimitLayer,
};
use security::{sanitize_error, validate_did, validate_content_id, validate_requirement, MAX_PAYLOAD_SIZE};
use tower::limit::ConcurrencyLimitLayer;
use cli::{Cli, Commands};
use identity::IdentityManager;
use messaging::MessagingManager;
use protocols::{ProtocolManager, ProofOfView};
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
            error!("Failed to generate DID: {}", sanitize_error(&e.to_string()));
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
/// SECURITY: Validates requirement string to prevent injection
async fn verify_requirement(
    State(state): State<AppState>,
    Json(req): Json<VerifyRequirementRequest>,
) -> Result<Json<VerifyRequirementResponse>, StatusCode> {
    // Validate requirement string
    if let Err(_) = validate_requirement(&req.requirement) {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Validate credential length
    if req.credential.len() > 10_000 {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    match state
        .protocols
        .verify_requirement(&req.credential, &req.requirement)
        .await
    {
        Ok(verified) => Ok(Json(VerifyRequirementResponse { verified })),
        Err(e) => {
            error!("Failed to verify requirement: {}", sanitize_error(&e.to_string()));
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// API request to issue a credential
/// SECURITY: Uses issuer_service_id instead of private key to load from keychain
#[derive(Deserialize)]
struct IssueCredentialRequest {
    subject_did: String,
    claims: serde_json::Value,
    issuer_did: String,
    /// Service ID to load issuer's private key from keychain (instead of transmitting key)
    issuer_service_id: String,
    credential_type: String,
    expiration_days: Option<u32>,
}

/// API response with issued credential
#[derive(Serialize)]
struct IssueCredentialResponse {
    credential: String,
    issuer_did: String,
    subject_did: String,
    credential_type: String,
}

/// Issue credential endpoint
/// SECURITY: Validates inputs and uses keychain instead of transmitted keys
async fn issue_credential(
    State(state): State<AppState>,
    Json(req): Json<IssueCredentialRequest>,
) -> Result<Json<IssueCredentialResponse>, StatusCode> {
    // Validate inputs
    if !validate_did(&req.subject_did) {
        return Err(StatusCode::BAD_REQUEST);
    }
    if !validate_did(&req.issuer_did) {
        return Err(StatusCode::BAD_REQUEST);
    }
    if req.issuer_service_id.is_empty() || req.issuer_service_id.len() > 100 {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // SECURITY: Load issuer's private key from keychain using service_id
    // NOTE: This requires TDK keychain integration. For now, API credential issuance
    // is disabled to prevent private key transmission. Use CLI with --issuer-key file for credential issuance.
    // TODO: Implement full TDK keychain key retrieval
    error!("API credential issuance requires keychain integration. Use CLI instead.");
    return Err(StatusCode::NOT_IMPLEMENTED);
    
    // Future implementation (when TDK keychain key retrieval is available):
    // let issuer_key = state.identity
    //     .get_issuer_private_key(&req.issuer_service_id, &req.issuer_did)
    //     .await
    //     .map_err(|e| {
    //         error!("Failed to load issuer key: {}", sanitize_error(&e.to_string()));
    //         StatusCode::UNAUTHORIZED
    //     })?;
    //
    // match state
    //     .protocols
    //     .issue_credential(
    //         &req.subject_did,
    //         req.claims,
    //         Some(&req.issuer_did),
    //         &issuer_key,
    //         &req.credential_type,
    //         req.expiration_days,
    //     )
    //     .await
    //     {
    //     Ok(credential) => {
    //         Ok(Json(IssueCredentialResponse {
    //             credential,
    //             issuer_did: req.issuer_did,
    //             subject_did: req.subject_did,
    //             credential_type: req.credential_type,
    //         }))
    //     }
    //     Err(e) => {
    //         error!("Failed to issue credential: {}", sanitize_error(&e.to_string()));
    //         Err(StatusCode::INTERNAL_SERVER_ERROR)
    //     }
    // }
}

/// API request to generate proof of view
#[derive(Deserialize)]
struct GenerateProofOfViewRequest {
    viewer_did: String,
    content_id: String,
    credential_proof: Option<String>,
}

/// API response with proof of view
#[derive(Serialize)]
struct GenerateProofOfViewResponse {
    proof: ProofOfView,
}

/// Generate proof of view endpoint
/// SECURITY: Validates inputs
async fn generate_proof_of_view(
    State(state): State<AppState>,
    Json(req): Json<GenerateProofOfViewRequest>,
) -> Result<Json<GenerateProofOfViewResponse>, StatusCode> {
    // Validate inputs
    if !validate_did(&req.viewer_did) {
        return Err(StatusCode::BAD_REQUEST);
    }
    if !validate_content_id(&req.content_id) {
        return Err(StatusCode::BAD_REQUEST);
    }
    if let Some(ref cred_proof) = req.credential_proof {
        if let Err(_) = validate_requirement(cred_proof) {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    
    match state
        .protocols
        .generate_proof_of_view(
            &req.viewer_did,
            &req.content_id,
            req.credential_proof.as_deref(),
        )
        .await
    {
        Ok(proof) => Ok(Json(GenerateProofOfViewResponse { proof })),
        Err(e) => {
            error!("Failed to generate proof of view: {}", sanitize_error(&e.to_string()));
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// API request to verify proof of view
#[derive(Deserialize)]
struct VerifyProofOfViewRequest {
    proof: ProofOfView,
}

/// API response with verification result
#[derive(Serialize)]
struct VerifyProofOfViewResponse {
    verified: bool,
}

/// Verify proof of view endpoint
async fn verify_proof_of_view(
    State(state): State<AppState>,
    Json(req): Json<VerifyProofOfViewRequest>,
) -> Result<Json<VerifyProofOfViewResponse>, StatusCode> {
    match state.protocols.verify_proof_of_view(&req.proof).await {
        Ok(verified) => Ok(Json(VerifyProofOfViewResponse { verified })),
        Err(e) => {
            error!("Failed to verify proof of view: {}", sanitize_error(&e.to_string()));
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Partner integration: Ad network bid request
#[derive(Deserialize)]
struct AdBidRequest {
    ad_slot_id: String,
    required_credentials: Vec<String>, // e.g., ["age > 18", "interest = crypto"]
    max_bid: Option<f64>,
}

/// Partner integration: Ad network bid response
#[derive(Serialize)]
struct AdBidResponse {
    bid: f64,
    proof_required: bool,
    message: String,
}

/// Ad network bid endpoint (partner integration)
async fn ad_bid(
    State(state): State<AppState>,
    Json(req): Json<AdBidRequest>,
) -> Result<Json<AdBidResponse>, StatusCode> {
    info!("Ad bid request for slot: {}", req.ad_slot_id);
    
    // This is a placeholder for partner integration
    // In production, this would verify credentials and return appropriate bids
    Ok(Json(AdBidResponse {
        bid: req.max_bid.unwrap_or(0.50),
        proof_required: !req.required_credentials.is_empty(),
        message: format!(
            "Bid for slot {} with {} credential requirements",
            req.ad_slot_id,
            req.required_credentials.len()
        ),
    }))
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

    // SECURITY: Configure CORS with restricted origins
    // Note: For development, allow localhost origins. For production, configure specific origins.
    use axum::http::{HeaderName, header};
    use tower_http::cors::AllowOrigin;
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact("http://localhost:8080".parse().unwrap()))
        .allow_origin(AllowOrigin::exact("http://127.0.0.1:8080".parse().unwrap()))
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
        ]);
    
    // SECURITY: Apply rate limiting and payload size limits
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/did/generate", post(generate_did))
        .route("/api/v1/protocols/verify", post(verify_requirement))
        .route("/api/v1/credentials/issue", post(issue_credential))
        .route("/api/v1/proof/view/generate", post(generate_proof_of_view))
        .route("/api/v1/proof/view/verify", post(verify_proof_of_view))
        .route("/api/v1/partner/ad/bid", post(ad_bid))
        .layer(
            ServiceBuilder::new()
                .layer(RequestBodyLimitLayer::new(MAX_PAYLOAD_SIZE))
                .layer(ConcurrencyLimitLayer::new(100)) // Limit concurrent requests
                .layer(cors)
        )
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
    info!("  POST /api/v1/credentials/issue - Issue a verifiable credential");
    info!("  POST /api/v1/proof/view/generate - Generate proof of view (Ad Tech)");
    info!("  POST /api/v1/proof/view/verify - Verify proof of view");
    info!("  POST /api/v1/partner/ad/bid - Ad network bid endpoint (Partner integration)");
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
        Some(Commands::IssueCredential {
            subject_did,
            claims,
            issuer_did,
            issuer_key,
            credential_type,
            expiration_days,
        }) => {
            info!("Issuing credential...");

            // Attempt to read claims from file, otherwise treat as JSON string
            let claims_str = if fs::metadata(&claims).is_ok() {
                fs::read_to_string(&claims)?
            } else {
                claims
            };

            let claims_json: serde_json::Value = serde_json::from_str(&claims_str)
                .map_err(|e| format!("Invalid JSON claims: {}", e))?;

            // Read issuer private key
            let key_str = fs::read_to_string(&issuer_key)
                .map_err(|e| format!("Failed to read issuer key file: {}", e))?;

            let protocol = ProtocolManager::new();
            let credential = protocol
                .issue_credential(
                    &subject_did,
                    claims_json,
                    Some(&issuer_did),
                    &key_str,
                    &credential_type,
                    Some(expiration_days),
                )
                .await?;

            println!("Credential issued successfully:");
            println!("{}", credential);
            Ok(())
        }
        Some(Commands::GenerateProofOfView {
            viewer_did,
            content_id,
            credential_proof,
        }) => {
            info!("Generating proof of view...");

            let protocol = ProtocolManager::new();
            let proof = protocol
                .generate_proof_of_view(&viewer_did, &content_id, credential_proof.as_deref())
                .await?;

            println!("Proof of view generated:");
            println!("{}", serde_json::to_string_pretty(&proof)?);
            Ok(())
        }
        None => run_server("127.0.0.1".to_string(), 3000).await,
    }
}
