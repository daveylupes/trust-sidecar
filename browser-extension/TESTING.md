# Browser Extension Testing Guide

## Prerequisites

1. **Trust Sidecar Server Running**
   ```bash
   cd trust-sidecar
   cargo run --release
   ```
   The server should be running on `http://127.0.0.1:3000`

2. **Browser**: Chrome, Edge, or any Chromium-based browser

## Loading the Extension

### Step 1: Open Extension Management
1. Open Chrome/Edge
2. Navigate to `chrome://extensions/` (or `edge://extensions/`)
3. Enable **Developer mode** (toggle in top-right corner)

### Step 2: Load Unpacked Extension
1. Click **"Load unpacked"** button
2. Navigate to the repository root directory
3. Select the `browser-extension` folder and click **"Select"**

### Step 3: Verify Installation
- You should see "Trust Sidecar" extension in the list
- The extension icon should appear in your browser toolbar
- Click the icon to open the popup and verify it shows connection status

## Testing the Extension

### Test 1: Extension Popup
1. Click the Trust Sidecar icon in the toolbar
2. The popup should show:
   - Connection status (Connected/Disconnected)
   - Your browser's DID
   - Refresh button

### Test 2: Test Page
1. Start the test server:
   ```bash
   cd browser-extension
   ./start-test-server.sh
   ```
2. Open in browser: `http://localhost:8080/test-page.html`

2. On the test page, try:
   - **Check Status**: Verify extension is loaded
   - **Get DID**: Get your browser's DID
   - **Generate Proof**: Create a proof of view
   - **Verify Credential**: Test credential verification
   - **Ad Slot Detection**: See automatic proof generation

### Test 3: Console Testing
1. Open any webpage
2. Open Developer Tools (F12)
3. Go to Console tab
4. Try these commands:

```javascript
// Check if extension is loaded
typeof window.trustSidecar

// Generate a proof
await window.trustSidecar.generateProof('https://example.com/test')

// Verify a credential
await window.trustSidecar.verifyCredential('{"age": 25}', 'age > 18')
```

### Test 4: Ad Slot Detection
1. Open any webpage with ad slots
2. The extension should automatically:
   - Detect ad slots (elements with class `ad-slot`, `advertisement`, or `data-ad-slot`)
   - Generate proofs
   - Store proofs in `data-proof-of-view` attribute
   - Dispatch `trustSidecarProofGenerated` events

3. Check in console:
```javascript
// Listen for proof events
document.addEventListener('trustSidecarProofGenerated', (e) => {
  console.log('Proof generated:', e.detail);
});
```

## Troubleshooting

### Extension Not Loading
- Check that all files are in the `browser-extension` directory
- Verify `manifest.json` is valid JSON
- Check browser console for errors (F12 → Console)

### Connection Failed
- Ensure Trust Sidecar server is running: `curl http://127.0.0.1:3000/health`
- Check browser console for CORS errors
- Verify `host_permissions` in manifest.json includes `http://127.0.0.1:3000/*`

### API Not Available
- Check background script is running:
  - Go to `chrome://extensions/`
  - Click "Service worker" link under Trust Sidecar
  - Check for errors in the service worker console

### DID Not Generated
- Check server is accessible from browser
- Verify storage permissions in manifest.json
- Check browser console for errors

## Expected Behavior

 **Working Correctly:**
- Extension popup shows "Connected" status
- DID is generated and displayed
- Proofs can be generated
- Credentials can be verified
- Ad slots trigger automatic proof generation

 **Issues to Report:**
- Extension fails to load
- Connection status shows "Disconnected"
- API calls fail with errors
- Proofs are not generated
- Events are not dispatched

## Next Steps

After successful testing:
1. Test with real ad networks
2. Integrate with ad bidding systems
3. Test credential issuance flow
4. Performance testing with multiple ad slots



