// Injected script that runs in page context
// This file is injected into pages to expose window.trustSidecar API

(function() {
  // Create message bridge between page context and content script
  window.trustSidecar = {
    async generateProof(contentId, credentialProof = null) {
      return new Promise((resolve, reject) => {
        const messageId = 'trustSidecar_' + Date.now() + '_' + Math.random();
        
        // Listen for response
        // SECURITY: Validate message origin to prevent XSS
        const listener = (event) => {
          // SECURITY: Only accept messages from same origin
          if (event.origin !== window.location.origin) {
            return; // Ignore messages from different origins
          }
          
          // Check if this is our response
          if (event.data && event.data.source === 'trustSidecarContent' && event.data.messageId === messageId) {
            window.removeEventListener('message', listener);
            if (event.data.error) {
              // Proof generation failed
              reject(new Error(event.data.error));
            } else {
              // Proof generated successfully
              resolve(event.data.result);
            }
          }
        };
        window.addEventListener('message', listener);
        
        // SECURITY: Send request to content script with origin validation
        window.postMessage({
          source: 'trustSidecarPage',
          messageId: messageId,
          action: 'generateProof',
          contentId: contentId,
          credentialProof: credentialProof
        }, window.location.origin);
        
        // Timeout after 10 seconds
        setTimeout(() => {
          window.removeEventListener('message', listener);
          reject(new Error('Request timeout - no response from content script'));
        }, 10000);
      });
    },
    
    async verifyCredential(credential, requirement) {
      return new Promise((resolve, reject) => {
        const messageId = 'trustSidecar_' + Date.now() + '_' + Math.random();
        
        const listener = (event) => {
          if (event.data && event.data.messageId === messageId) {
            window.removeEventListener('message', listener);
            if (event.data.error) {
              reject(new Error(event.data.error));
            } else {
              resolve(event.data.result);
            }
          }
        };
        window.addEventListener('message', listener);
        
        window.postMessage({
          source: 'trustSidecarPage',
          messageId: messageId,
          action: 'verifyCredential',
          credential: credential,
          requirement: requirement
        }, window.location.origin);
        
        setTimeout(() => {
          window.removeEventListener('message', listener);
          reject(new Error('Request timeout'));
        }, 10000);
      });
    },
    
    async getDid() {
      return new Promise((resolve, reject) => {
        const messageId = 'trustSidecar_' + Date.now() + '_' + Math.random();
        
        // SECURITY: Validate message origin
        const listener = (event) => {
          if (event.origin !== window.location.origin) {
            return;
          }
          if (event.data && event.data.messageId === messageId) {
            window.removeEventListener('message', listener);
            if (event.data.error) {
              reject(new Error(event.data.error));
            } else {
              resolve(event.data.result);
            }
          }
        };
        window.addEventListener('message', listener);
        
        window.postMessage({
          source: 'trustSidecarPage',
          messageId: messageId,
          action: 'getDid'
        }, window.location.origin);
        
        setTimeout(() => {
          window.removeEventListener('message', listener);
          reject(new Error('Request timeout'));
        }, 10000);
      });
    }
  };
  
  // Trust Sidecar API injected into page context
})();
