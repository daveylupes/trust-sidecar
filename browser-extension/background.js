// Background service worker for Trust Sidecar browser extension
// Handles communication with the Trust Sidecar API server

const SIDECAR_API_URL = 'http://127.0.0.1:3000';

// Initialize extension
chrome.runtime.onInstalled.addListener(() => {
  console.log('Trust Sidecar extension installed');
  
  // Check if sidecar is running
  checkSidecarHealth();
});

// Listen for messages from content scripts
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.action === 'generateProofOfView') {
    generateProofOfView(request.viewerDid, request.contentId, request.credentialProof)
      .then(proof => sendResponse({ success: true, proof }))
      .catch(error => sendResponse({ success: false, error: error.message }));
    return true; // Keep channel open for async response
  }
  
  if (request.action === 'getDid') {
    getOrCreateDid()
      .then(did => sendResponse({ success: true, did }))
      .catch(error => sendResponse({ success: false, error: error.message }));
    return true;
  }
  
  if (request.action === 'verifyCredential') {
    verifyCredential(request.credential, request.requirement)
      .then(result => sendResponse({ success: true, verified: result }))
      .catch(error => sendResponse({ success: false, error: error.message }));
    return true;
  }
});

// Check if Trust Sidecar API is running
async function checkSidecarHealth() {
  try {
    const response = await fetch(`${SIDECAR_API_URL}/health`);
    const data = await response.json();
    console.log('Trust Sidecar is running:', data);
    return true;
  } catch (error) {
    console.warn('Trust Sidecar API not available:', error);
    return false;
  }
}

// SECURITY: Simple encryption for DID storage (XOR cipher for basic obfuscation)
// In production, use proper encryption with Web Crypto API
function encryptDid(did) {
  // Simple obfuscation - in production, use Web Crypto API
  const key = 'trust-sidecar-key-2024';
  let encrypted = '';
  for (let i = 0; i < did.length; i++) {
    encrypted += String.fromCharCode(did.charCodeAt(i) ^ key.charCodeAt(i % key.length));
  }
  return btoa(encrypted);
}

function decryptDid(encrypted) {
  try {
    const encryptedData = atob(encrypted);
    const key = 'trust-sidecar-key-2024';
    let decrypted = '';
    for (let i = 0; i < encryptedData.length; i++) {
      decrypted += String.fromCharCode(encryptedData.charCodeAt(i) ^ key.charCodeAt(i % key.length));
    }
    return decrypted;
  } catch (e) {
    return null;
  }
}

// Get or create a DID for this browser
// SECURITY: Encrypt DID before storing
async function getOrCreateDid() {
  // Check storage first
  const result = await chrome.storage.local.get(['browserDidEncrypted']);
  if (result.browserDidEncrypted) {
    const decrypted = decryptDid(result.browserDidEncrypted);
    if (decrypted) {
      return decrypted;
    }
    // If decryption fails, clear and regenerate
    await chrome.storage.local.remove(['browserDidEncrypted']);
  }
  
  // Generate new DID
  try {
    const response = await fetch(`${SIDECAR_API_URL}/api/v1/did/generate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ service_id: 'browser-extension' })
    });
    
    if (!response.ok) {
      throw new Error('Failed to generate DID');
    }
    
    const data = await response.json();
    const did = data.did;
    
    // SECURITY: Encrypt DID before storing
    const encrypted = encryptDid(did);
    await chrome.storage.local.set({ browserDidEncrypted: encrypted });
    
    return did;
  } catch (error) {
    throw new Error(`Failed to get/create DID: ${error.message}`);
  }
}

// Generate proof of view for ad tech
async function generateProofOfView(viewerDid, contentId, credentialProof = null) {
  try {
    const response = await fetch(`${SIDECAR_API_URL}/api/v1/proof/view/generate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        viewer_did: viewerDid,
        content_id: contentId,
        credential_proof: credentialProof
      })
    });
    
    if (!response.ok) {
      throw new Error('Failed to generate proof of view');
    }
    
    const data = await response.json();
    return data.proof;
  } catch (error) {
    throw new Error(`Failed to generate proof: ${error.message}`);
  }
}

// Verify a credential requirement
async function verifyCredential(credential, requirement) {
  try {
    const response = await fetch(`${SIDECAR_API_URL}/api/v1/protocols/verify`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        credential: credential,
        requirement: requirement
      })
    });
    
    if (!response.ok) {
      throw new Error('Failed to verify credential');
    }
    
    const data = await response.json();
    return data.verified;
  } catch (error) {
    throw new Error(`Failed to verify credential: ${error.message}`);
  }
}
