//! Security Module
//!
//! Provides security utilities: input validation, rate limiting, error sanitization

use std::time::Duration;
use validator::ValidationError;

/// Maximum JSON payload size (1MB)
pub const MAX_PAYLOAD_SIZE: usize = 1_048_576;

/// Rate limit configuration
pub const RATE_LIMIT_REQUESTS: u32 = 100;
pub const RATE_LIMIT_DURATION: Duration = Duration::from_secs(60);

/// Rate limiting configuration
/// Note: Rate limiting is applied via tower-http middleware in main.rs
/// This function provides configuration constants
pub fn get_rate_limit_config() -> (u32, Duration) {
    (RATE_LIMIT_REQUESTS, RATE_LIMIT_DURATION)
}

/// Sanitize error messages to prevent information disclosure
pub fn sanitize_error(error: &str) -> String {
    // Remove file paths, line numbers, and internal details
    let sanitized = error
        .lines()
        .take(1) // Only first line
        .collect::<Vec<_>>()
        .join(" ");
    
    // Generic error message for production
    if sanitized.contains("private key") || sanitized.contains("secret") {
        "Invalid request".to_string()
    } else if sanitized.contains("parse") || sanitized.contains("decode") {
        "Invalid input format".to_string()
    } else {
        sanitized
    }
}

/// Validate requirement string to prevent injection
pub fn validate_requirement(requirement: &str) -> Result<(), ValidationError> {
    // Whitelist allowed characters and patterns
    let allowed_chars = requirement.chars().all(|c| {
        c.is_alphanumeric() || 
        c.is_whitespace() || 
        "><=!&|()".contains(c)
    });
    
    if !allowed_chars {
        return Err(ValidationError::new("invalid_characters"));
    }
    
    // Check for dangerous patterns
    let dangerous = ["eval", "exec", "import", "require", "script", "javascript"];
    let lower = requirement.to_lowercase();
    if dangerous.iter().any(|&pattern| lower.contains(pattern)) {
        return Err(ValidationError::new("dangerous_pattern"));
    }
    
    // Check length
    if requirement.len() > 256 {
        return Err(ValidationError::new("too_long"));
    }
    
    Ok(())
}

/// Validate DID format
pub fn validate_did(did: &str) -> bool {
    did.starts_with("did:") && did.len() < 200
}

/// Validate content ID (URL or identifier)
pub fn validate_content_id(content_id: &str) -> bool {
    content_id.len() < 2048 && 
    (content_id.starts_with("http://") || 
     content_id.starts_with("https://") || 
     content_id.chars().all(|c| c.is_alphanumeric() || "-_./".contains(c)))
}
