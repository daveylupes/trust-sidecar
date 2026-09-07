# Trust Sidecar Browser Extension

Browser extension for Trust Sidecar that enables proof-of-view generation and credential verification in web browsers.

## Features

- **Automatic Proof Generation**: Automatically generates proof-of-view for ad slots
- **DID Management**: Creates and manages a DID for the browser
- **Credential Verification**: Verifies credentials without revealing raw data
- **Ad Tech Integration**: Provides APIs for ad networks to request proofs

## Installation

### Prerequisites

- Trust Sidecar server running on `http://127.0.0.1:3000`
- Chrome 88+ or Edge 88+ (Manifest V3 required)

### Loading the Extension

1. **Build and start the Trust Sidecar server** (see main [README](../README.md))
   ```bash
   cargo run
   ```

2. **Load the extension in Chrome/Edge**:
   - Open `chrome://extensions/` (or `edge://extensions/`)
   - Enable **"Developer mode"** (toggle in top-right corner)
   - Click **"Load unpacked"**
   - Select the `browser-extension` directory (not the parent directory)

3. **Verify installation**:
   - Look for "Trust Sidecar" in the extensions list
   - Check for the extension icon in your toolbar
   - Click the icon to see connection status

**Important**: Select the `browser-extension` folder itself, not the parent directory. The `manifest.json` file must be directly in the selected folder.

## Usage

### For Websites

The extension exposes a global `window.trustSidecar` API:

```javascript
// Generate a proof of view
const proof = await window.trustSidecar.generateProof('https://example.com/article');

// Verify a credential requirement
const verified = await window.trustSidecar.verifyCredential(credential, 'age > 18');

// Get the browser's DID
const did = await window.trustSidecar.getDid();
```

### For Ad Networks

Listen for the `trustSidecarProofGenerated` event:

```javascript
document.addEventListener('trustSidecarProofGenerated', (event) => {
  const { adSlotId, proof } = event.detail;
  // Use proof for ad bidding
  sendBidRequest(adSlotId, proof);
});
```

See [ad-network-integration.js](ad-network-integration.js) for a complete integration example.

## Testing

### Quick Test

1. Start the Trust Sidecar server: `cargo run`
2. Start the test server: `./start-test-server.sh`
3. Open `http://localhost:8080/test-page.html`
4. Click "Check Status" - should show "Extension loaded and working!"

### Demo Page

For ad network integration testing:

1. Start all services: `../start-all.sh`
2. Open `http://localhost:8080/ad-network-demo.html`
3. Ad slots will automatically generate proofs and receive bids

## Development

### Project Structure

```
browser-extension/
├── manifest.json           # Extension configuration (Manifest V3)
├── background.js           # Service worker (communicates with API)
├── content.js             # Content script (runs on web pages)
├── injected.js            # Script injected into page context
├── popup.html/js          # Extension popup UI
├── ad-network-integration.js  # Ad network integration library
├── test-page.html         # Test page for manual testing
├── ad-network-demo.html   # Demo page for ad network integration
└── icons/                 # Extension icons
```

### Key Components

- **background.js**: Service worker that handles API communication
- **content.js**: Content script that detects ad slots and generates proofs
- **injected.js**: Script injected into page context to expose `window.trustSidecar` API
- **ad-network-integration.js**: Integration library for ad networks

### Reloading After Changes

After modifying extension files:

1. Go to `chrome://extensions/`
2. Click the reload icon on the Trust Sidecar card
3. Refresh the test page (Ctrl+R or Cmd+R)

**Note**: Changes to `manifest.json` require removing and re-adding the extension.

## Configuration

### Server URL

The extension connects to `http://127.0.0.1:3000` by default. To change this:

1. Edit `background.js`
2. Update the `SIDECAR_API_URL` constant
3. Reload the extension

### Permissions

The extension requires:
- `storage`: For DID storage
- `activeTab`: For content script injection
- `scripting`: For script injection
- Host permissions: For API communication

## Troubleshooting

See [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for common issues and solutions.

Common issues:
- Extension not detected: Check it's loaded and enabled in `chrome://extensions/`
- API not available: Reload extension and refresh page
- Connection failed: Verify server is running on `http://127.0.0.1:3000`

## Documentation

- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Common issues and solutions
- [HOW_TO_LOAD.md](HOW_TO_LOAD.md) - Detailed loading instructions
- [QUICK_START.md](QUICK_START.md) - Quick setup guide
- [TESTING.md](TESTING.md) - Testing guidelines

## Notes

- The extension requires the Trust Sidecar server to be running
- DIDs are stored in browser local storage
- Proofs are automatically generated for detected ad slots with `data-ad-slot` attribute
- The extension uses Manifest V3 (Chrome 88+ required)
