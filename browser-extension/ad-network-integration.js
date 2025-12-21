// Ad Network Integration Example
// This shows how to integrate Trust Sidecar with an ad network

/**
 * Send bid request to ad network with proof of view
 * @param {string} adSlotId - The ad slot identifier
 * @param {string} url - The page URL
 * @param {Array<string>} requiredCredentials - Required credential requirements
 * @param {string} adNetworkUrl - Ad network API URL (default: http://127.0.0.1:8081)
 * @returns {Promise<Object>} Bid response from ad network
 */
async function sendBidRequest(adSlotId, url, requiredCredentials = [], adNetworkUrl = 'http://127.0.0.1:8081', proofOfView = null) {
  // Generate proof of view if not provided
  if (!proofOfView && typeof window.trustSidecar !== 'undefined') {
    try {
    proofOfView = await window.trustSidecar.generateProof(url);
    } catch (error) {
      console.warn('Failed to generate proof:', error);
      // Continue without proof (ad network will handle it)
    }
  }
  
  // Prepare bid request
  const bidRequest = {
    ad_slot_id: adSlotId,
    url: url,
    required_credentials: requiredCredentials,
    proof_of_view: proofOfView
  };
  
  // Prepare and send bid request
  
  // Send bid request to ad network
  try {
    const response = await fetch(`${adNetworkUrl}/api/v1/bid`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(bidRequest)
    });
    
    if (!response.ok) {
      throw new Error(`Ad network error: ${response.status}`);
    }
    
    const bidResponse = await response.json();
    
    return bidResponse;
  } catch (error) {
    console.error('Bid request failed:', error);
    throw error;
  }
}

/**
 * Ad Network Integration Class
 */
class AdNetworkIntegration {
  constructor(adNetworkUrl = 'http://127.0.0.1:8081') {
    this.adNetworkUrl = adNetworkUrl;
    this.bids = new Map();
  }
  
  /**
   * Request bid for an ad slot
   */
  async requestBid(adSlotId, url, requiredCredentials = [], proofOfView = null) {
    try {
      // If no proof provided, try to get it from the ad slot or generate it
      if (!proofOfView) {
        const adSlot = document.querySelector(`[data-ad-slot="${adSlotId}"]`);
        if (adSlot) {
          const proofData = adSlot.getAttribute('data-proof-of-view');
          if (proofData) {
            try {
            proofOfView = JSON.parse(proofData);
            } catch (e) {
              console.warn('Failed to parse stored proof, generating new one');
            }
          }
        }
        
        // If still no proof and Trust Sidecar is available, generate one
        if (!proofOfView && typeof window.trustSidecar !== 'undefined') {
          try {
            proofOfView = await window.trustSidecar.generateProof(url);
            // Generated new proof for bid request
            if (!proofOfView) {
              // generateProof returned undefined/null
              throw new Error('Proof generation returned no result');
            }
            // Store it on the ad slot for future use
            const adSlot = document.querySelector(`[data-ad-slot="${adSlotId}"]`);
            if (adSlot) {
              adSlot.setAttribute('data-proof-of-view', JSON.stringify(proofOfView));
            }
          } catch (error) {
            console.error('Failed to generate proof:', error);
          }
        }
      }
      
      const bidResponse = await sendBidRequest(adSlotId, url, requiredCredentials, this.adNetworkUrl, proofOfView);
      
      // Store bid
      this.bids.set(adSlotId, bidResponse);
      
      // Dispatch event for ad rendering
      const event = new CustomEvent('trustSidecarBidReceived', {
        detail: {
          adSlotId: adSlotId,
          bid: bidResponse
        }
      });
      document.dispatchEvent(event);
      
      return bidResponse;
    } catch (error) {
      console.error('Bid request failed:', error);
      return null;
    }
  }
  
  /**
   * Get statistics from ad network
   */
  async getStats() {
    try {
      const response = await fetch(`${this.adNetworkUrl}/api/v1/stats`);
      if (!response.ok) {
        throw new Error(`Stats request failed: ${response.status}`);
      }
      return await response.json();
    } catch (error) {
      console.error('Stats request failed:', error);
      return null;
    }
  }
}

// Export for use in other scripts
if (typeof window !== 'undefined') {
  window.AdNetworkIntegration = AdNetworkIntegration;
  window.sendBidRequest = sendBidRequest;
}

// Auto-integration: Listen for ad slots and automatically send bids
function initializeAdNetworkIntegration() {
  if (typeof window.trustSidecar === 'undefined') {
    // Wait for trustSidecar to be available
    setTimeout(initializeAdNetworkIntegration, 500);
    return;
  }
  
  const adNetwork = new AdNetworkIntegration();
  window.adNetworkIntegration = adNetwork; // Make it globally available
  
  // Listen for proof generation events
  document.addEventListener('trustSidecarProofGenerated', async (event) => {
    const { adSlotId, proof } = event.detail;
    
    // Mark slot as loading
    const adSlot = document.querySelector(`[data-ad-slot="${adSlotId}"]`);
    if (adSlot) {
      adSlot.classList.add('loading');
    }
    
    // Extract credential requirements from ad slot
    const requiredCredentials = adSlot?.getAttribute('data-required-credentials')?.split(',').map(c => c.trim()) || [];
    
    // Send bid request WITH the proof
    try {
      const bidResponse = await adNetwork.requestBid(adSlotId, window.location.href, requiredCredentials, proof);
      if (adSlot) {
        adSlot.classList.remove('loading');
      }
      // Bid request completed
    } catch (error) {
      console.error('Bid request failed:', error);
      if (adSlot) {
        adSlot.classList.remove('loading');
      }
    }
  });
  
  // Also manually trigger bids for existing ad slots on page
  // This handles cases where proofs were generated before the listener was set up
  setTimeout(async () => {
    const adSlots = document.querySelectorAll('[data-ad-slot]');
    
    for (const adSlot of adSlots) {
      const adSlotId = adSlot.getAttribute('data-ad-slot');
      const proofData = adSlot.getAttribute('data-proof-of-view');
      
      if (proofData) {
        try {
          const proof = JSON.parse(proofData);
          
          // Trigger the event to process it
          const event = new CustomEvent('trustSidecarProofGenerated', {
            detail: {
              adSlotId: adSlotId,
              proof: proof
            }
          });
          document.dispatchEvent(event);
        } catch (e) {
          console.error('Failed to parse proof data:', e);
        }
      } else {
        // No proof yet, generate one first
        try {
          if (typeof window.trustSidecar !== 'undefined') {
            const proof = await window.trustSidecar.generateProof(window.location.href);
            
            // Store proof on the ad slot
            adSlot.setAttribute('data-proof-of-view', JSON.stringify(proof));
            
            // Trigger the event to process it
            const event = new CustomEvent('trustSidecarProofGenerated', {
              detail: {
                adSlotId: adSlotId,
                proof: proof
              }
            });
            document.dispatchEvent(event);
          } else {
            console.warn('Trust Sidecar not available, sending bid without proof');
            const requiredCredentials = adSlot.getAttribute('data-required-credentials')?.split(',').map(c => c.trim()) || [];
            await adNetwork.requestBid(adSlotId, window.location.href, requiredCredentials);
          }
        } catch (error) {
          console.error(`Failed to generate proof for ${adSlotId}:`, error);
          // Fallback: send bid without proof
          const requiredCredentials = adSlot.getAttribute('data-required-credentials')?.split(',').map(c => c.trim()) || [];
          await adNetwork.requestBid(adSlotId, window.location.href, requiredCredentials);
        }
      }
    }
  }, 2000);
  
  // Ad Network Integration initialized
}

// Start initialization
if (typeof window !== 'undefined') {
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initializeAdNetworkIntegration);
  } else {
    initializeAdNetworkIntegration();
  }
}
