// Content script for Trust Sidecar browser extension
// Intercepts ad requests and generates proof-of-view

(function() {
  'use strict';
  
  // Content script initialization
  // Note: Console logs are minimal for production; enable debug logging if needed
  
  // Inject script into page context using script tag with extension URL
  // This avoids CSP violations by using a file instead of inline script
  function injectScript() {
    const script = document.createElement('script');
    script.src = chrome.runtime.getURL('injected.js');
    script.onload = function() {
      this.remove();
      // API injection script loaded successfully
    };
    script.onerror = function() {
      console.error('Failed to load injected.js');
      console.error('   Make sure injected.js is in web_accessible_resources');
    };
    
    const target = document.head || document.documentElement || document.body;
    if (target) {
      target.appendChild(script);
    } else {
      // If DOM not ready, wait a bit and try again
      setTimeout(injectScript, 10);
    }
  }
  
  // Try to inject immediately
  if (document.readyState === 'loading') {
    // DOM is still loading, inject when ready
    document.addEventListener('DOMContentLoaded', injectScript);
  } else {
    // DOM already loaded, inject now
    injectScript();
  }
  
  // Listen for messages from page context
  // SECURITY: Validate message origin to prevent XSS
  window.addEventListener('message', async (event) => {
    // SECURITY: Only accept messages from same origin (page context)
    // Content scripts run in isolated world, but we still validate origin
    if (event.origin !== window.location.origin) {
      console.warn('Rejected message from different origin:', event.origin);
      return;
    }
    
    // Only accept messages from our injected script
    if (event.data && event.data.source === 'trustSidecarPage') {
      const { messageId, action, contentId, credentialProof, credential, requirement } = event.data;
      
      try {
        let result;
        
        if (action === 'getDid') {
          // Get browser DID directly
          const didResponse = await chrome.runtime.sendMessage({ action: 'getDid' });
          if (!didResponse.success) {
            throw new Error(didResponse.error);
          }
          result = didResponse.did;
        } else if (action === 'generateProof') {
          // Get browser DID
          const didResponse = await chrome.runtime.sendMessage({ action: 'getDid' });
          if (!didResponse.success) {
            throw new Error(didResponse.error);
          }
          
          // Generate proof
          const proofResponse = await chrome.runtime.sendMessage({
            action: 'generateProofOfView',
            viewerDid: didResponse.did,
            contentId: contentId,
            credentialProof: credentialProof
          });
          
          if (!proofResponse.success) {
            throw new Error(proofResponse.error);
          }
          
          result = proofResponse.proof;
        } else if (action === 'verifyCredential') {
          const response = await chrome.runtime.sendMessage({
            action: 'verifyCredential',
            credential: credential,
            requirement: requirement
          });
          
          if (!response.success) {
            throw new Error(response.error);
          }
          
          result = response.verified;
        } else {
          throw new Error('Unknown action: ' + action);
        }
        
        // SECURITY: Send response back to page context with origin validation
        // Use window.location.origin instead of '*' to prevent XSS
        window.postMessage({
          source: 'trustSidecarContent',
          messageId: messageId,
          result: result
        }, window.location.origin);
      } catch (error) {
        // SECURITY: Send error back to page context with origin validation
        window.postMessage({
          source: 'trustSidecarContent',
          messageId: messageId,
          error: error.message
        }, window.location.origin);
      }
    }
  });
  
  // Listen for ad slot requests
  const observer = new MutationObserver((mutations) => {
    mutations.forEach((mutation) => {
      mutation.addedNodes.forEach((node) => {
        if (node.nodeType === 1) { // Element node
          // Check for ad slots
          if (node.classList && (
            node.classList.contains('ad-slot') ||
            node.classList.contains('advertisement') ||
            node.getAttribute('data-ad-slot')
          )) {
            handleAdSlot(node);
          }
        }
      });
    });
  });
  
  // Start observing when DOM is ready
  if (document.body) {
    observer.observe(document.body, {
      childList: true,
      subtree: true
    });
  } else {
    // Wait for body to be available
    const bodyObserver = new MutationObserver((mutations, obs) => {
      if (document.body) {
        observer.observe(document.body, {
          childList: true,
          subtree: true
        });
        obs.disconnect();
      }
    });
    bodyObserver.observe(document.documentElement, {
      childList: true,
      subtree: true
    });
  }
  
  // Handle ad slot
  async function handleAdSlot(adSlot) {
    const adSlotId = adSlot.getAttribute('data-ad-slot') || 
                     adSlot.getAttribute('id') || 
                     window.location.href;
    
    // Ad slot detected and processing
    
    // Get browser DID
    const didResponse = await chrome.runtime.sendMessage({ action: 'getDid' });
    if (!didResponse.success) {
      console.error('Failed to get DID:', didResponse.error);
      return;
    }
    
    const viewerDid = didResponse.did;
    
    // Generate proof of view
    const proofResponse = await chrome.runtime.sendMessage({
      action: 'generateProofOfView',
      viewerDid: viewerDid,
      contentId: adSlotId,
      credentialProof: null
    });
    
    if (proofResponse.success) {
      // Proof of view generated successfully
      
      // Store proof in ad slot data attribute
      adSlot.setAttribute('data-proof-of-view', JSON.stringify(proofResponse.proof));
      
      // Dispatch custom event for ad networks (in page context, not isolated world)
      // We need to inject a script into the page context to dispatch the event
      const script = document.createElement('script');
      script.textContent = `
        (function() {
          const event = new CustomEvent('trustSidecarProofGenerated', {
            detail: {
              adSlotId: ${JSON.stringify(adSlotId)},
              proof: ${JSON.stringify(proofResponse.proof)}
            }
          });
          window.dispatchEvent(event);
        })();
      `;
      (document.head || document.documentElement).appendChild(script);
      script.remove();
    } else {
      console.error('Failed to generate proof:', proofResponse.error);
    }
  }
  
})();
