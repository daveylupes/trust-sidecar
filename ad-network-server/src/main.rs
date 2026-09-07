//! Mock Ad Network Server for Testing Trust Sidecar Integration
//!
//! This server simulates an ad network that:
//! - Receives ad slot requests with credential requirements
//! - Verifies proofs of view
//! - Returns bids based on verified credentials

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

#[derive(Clone)]
struct AppState {
    // Store of verified proofs (in production, this would be a database)
    verified_proofs: Arc<RwLock<HashMap<String, ProofOfView>>>,
    // Statistics
    stats: Arc<RwLock<AdNetworkStats>>,
}

#[derive(Default)]
struct AdNetworkStats {
    total_requests: u64,
    verified_views: u64,
    total_bid_value: f64,
}

/// Proof of View from Trust Sidecar
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProofOfView {
    viewer_did: String,
    content_id: String,
    timestamp: i64,
    proof_hash: String,
    credential_proof: Option<String>,
}

/// Ad slot bid request
#[derive(Deserialize)]
struct BidRequest {
    ad_slot_id: String,
    url: String,
    required_credentials: Vec<String>,
    proof_of_view: Option<ProofOfView>,
}

/// Ad slot bid response
#[derive(Serialize)]
struct BidResponse {
    bid: f64,
    currency: String,
    ad_creative_id: Option<String>,
    verification_status: String,
    message: String,
}

/// Verify proof of view request
#[derive(Deserialize)]
struct VerifyProofRequest {
    proof: ProofOfView,
}

/// Verify proof of view response
#[derive(Serialize)]
struct VerifyProofResponse {
    verified: bool,
    message: String,
}

/// Statistics endpoint response
#[derive(Serialize)]
struct StatsResponse {
    total_requests: u64,
    verified_views: u64,
    total_bid_value: f64,
    average_bid: f64,
}

/// Health check endpoint
async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "mock-ad-network",
        "version": "1.0.0"
    }))
}

/// Ad slot bid endpoint
/// This simulates an ad network receiving a bid request with proof of view
async fn bid(
    State(state): State<AppState>,
    Json(req): Json<BidRequest>,
) -> Result<Json<BidResponse>, StatusCode> {
    info!("Bid request for slot: {}", req.ad_slot_id);
    
    let mut stats = state.stats.write().await;
    stats.total_requests += 1;
    
    // Verify proof of view if provided
    let verification_status = if let Some(proof) = &req.proof_of_view {
        // Verify proof with Trust Sidecar
        match verify_proof_with_sidecar(proof).await {
            Ok(verified) => {
                if verified {
                    stats.verified_views += 1;
                    let proof_key = format!("{}_{}", proof.viewer_did, proof.content_id);
                    state.verified_proofs.write().await.insert(proof_key, proof.clone());
                    "verified".to_string()
                } else {
                    "verification_failed".to_string()
                }
            }
            Err(e) => {
                warn!("Proof verification error: {}", e);
                "verification_error".to_string()
            }
        }
    } else {
        "no_proof".to_string()
    };
    
    // Calculate bid based on credentials and verification
    let bid = calculate_bid(&req.required_credentials, &verification_status);
    stats.total_bid_value += bid;
    
    let message = if verification_status == "verified" {
        format!(
            "Bid placed for slot {} with {} credential requirements. Proof verified.",
            req.ad_slot_id,
            req.required_credentials.len()
        )
    } else {
        format!(
            "Bid placed for slot {} but proof verification: {}",
            req.ad_slot_id, verification_status
        )
    };
    
    Ok(Json(BidResponse {
        bid,
        currency: "USD".to_string(),
        ad_creative_id: Some(format!("creative-{}", req.ad_slot_id)),
        verification_status,
        message,
    }))
}

/// Verify proof of view endpoint
async fn verify_proof(
    State(state): State<AppState>,
    Json(req): Json<VerifyProofRequest>,
) -> Result<Json<VerifyProofResponse>, StatusCode> {
    info!("Verifying proof of view...");
    
    match verify_proof_with_sidecar(&req.proof).await {
        Ok(verified) => {
            if verified {
                let proof_key = format!("{}_{}", req.proof.viewer_did, req.proof.content_id);
                state.verified_proofs.write().await.insert(proof_key, req.proof);
                
                Ok(Json(VerifyProofResponse {
                    verified: true,
                    message: "Proof verified successfully".to_string(),
                }))
            } else {
                Ok(Json(VerifyProofResponse {
                    verified: false,
                    message: "Proof verification failed".to_string(),
                }))
            }
        }
        Err(e) => {
            warn!("Proof verification error: {}", e);
            Ok(Json(VerifyProofResponse {
                verified: false,
                message: format!("Verification error: {}", e),
            }))
        }
    }
}

/// Statistics endpoint
async fn stats(State(state): State<AppState>) -> Json<StatsResponse> {
    let stats = state.stats.read().await;
    let average_bid = if stats.total_requests > 0 {
        stats.total_bid_value / stats.total_requests as f64
    } else {
        0.0
    };
    
    Json(StatsResponse {
        total_requests: stats.total_requests,
        verified_views: stats.verified_views,
        total_bid_value: stats.total_bid_value,
        average_bid,
    })
}

/// Verify proof with Trust Sidecar API
async fn verify_proof_with_sidecar(proof: &ProofOfView) -> Result<bool, String> {
    let client = reqwest::Client::new();
    
    let response = client
        .post("http://127.0.0.1:3000/api/v1/proof/view/verify")
        .json(&serde_json::json!({ "proof": proof }))
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Trust Sidecar: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Trust Sidecar returned error: {}", response.status()));
    }
    
    let result: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    result
        .get("verified")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid response format".to_string())
}

/// Calculate bid based on credentials and verification status
fn calculate_bid(required_credentials: &[String], verification_status: &str) -> f64 {
    let base_bid = 0.10;
    
    // Higher bid for verified proofs
    let verification_multiplier = if verification_status == "verified" {
        2.0
    } else {
        0.5
    };
    
    // Higher bid for more credential requirements (more targeted)
    let credential_multiplier = 1.0 + (required_credentials.len() as f64 * 0.1);
    
    base_bid * verification_multiplier * credential_multiplier
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let app_state = AppState {
        verified_proofs: Arc::new(RwLock::new(HashMap::new())),
        stats: Arc::new(RwLock::new(AdNetworkStats::default())),
    };
    
    // SECURITY: Configure CORS with restricted origins
    use axum::http::header;
    use tower_http::cors::AllowOrigin;
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact("http://localhost:8080".parse().unwrap()))
        .allow_origin(AllowOrigin::exact("http://127.0.0.1:8080".parse().unwrap()))
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers([header::CONTENT_TYPE]);
    
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/bid", post(bid))
        .route("/api/v1/verify", post(verify_proof))
        .route("/api/v1/stats", get(stats))
        .layer(cors)
        .with_state(app_state);
    
    let addr = "127.0.0.1:8081";
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    info!("Mock Ad Network Server listening on http://{}", addr);
    info!("Endpoints:");
    info!("  GET  /health - Health check");
    info!("  POST /api/v1/bid - Ad slot bid request");
    info!("  POST /api/v1/verify - Verify proof of view");
    info!("  GET  /api/v1/stats - Statistics");
    info!("");
    info!("Make sure Trust Sidecar is running on http://127.0.0.1:3000");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
