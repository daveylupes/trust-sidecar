// Popup script for Trust Sidecar browser extension

const SIDECAR_API_URL = 'http://127.0.0.1:3000';

async function init() {
  await checkConnection();
  await loadDid();
  
  document.getElementById('refresh').addEventListener('click', async () => {
    await checkConnection();
    await loadDid();
  });
}

async function checkConnection() {
  const statusEl = document.getElementById('status');
  
  try {
    const response = await fetch(`${SIDECAR_API_URL}/health`);
    const data = await response.json();
    
    statusEl.textContent = `Connected (v${data.version})`;
    statusEl.className = 'status connected';
  } catch (error) {
    statusEl.textContent = 'Disconnected - Start Trust Sidecar server';
    statusEl.className = 'status disconnected';
  }
}

async function loadDid() {
  const didEl = document.getElementById('did');
  
  try {
    const response = await chrome.runtime.sendMessage({ action: 'getDid' });
    if (response.success) {
      didEl.textContent = response.did;
    } else {
      didEl.textContent = 'Error: ' + response.error;
    }
  } catch (error) {
    didEl.textContent = 'Error loading DID';
  }
}

init();
