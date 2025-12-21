# Quick Start - Browser Extension Testing

## Fast Setup (3 Steps)

### Step 1: Ensure Server is Running
```bash
cd trust-sidecar
cargo run --release
```
Server should show: `Trust Sidecar API server listening on http://127.0.0.1:3000`

### Step 2: Load Extension in Browser

**Chrome/Edge:**
1. Open `chrome://extensions/` (or `edge://extensions/`)
2. Enable **Developer mode** (top-right toggle)
3. Click **"Load unpacked"**
4. Select the `browser-extension` directory
5. Extension should appear in your toolbar

### Step 3: Test It!

**Option A: Test via Popup**
- Click the Trust Sidecar icon in toolbar
- Should show "Connected" status and your DID

**Option B: Test via Test Page**
```bash
cd browser-extension
./start-test-server.sh
# Open http://localhost:8080/test-page.html
```

**Option C: Test via Console**
- Open any webpage
- Press F12 → Console
- Try:
```javascript
await window.trustSidecar.generateProof('https://example.com/test')
```

## Validation

Run the validation script to check everything:
```bash
cd browser-extension
./validate-extension.sh
```

## Troubleshooting

**Extension not loading?**
- Check `validate-extension.sh` output
- Look for errors in `chrome://extensions/` page

**Connection failed?**
- Verify server is running: `curl http://127.0.0.1:3000/health`
- Check browser console (F12) for errors

**API not available?**
- Check background script: Click "Service worker" link in extensions page
- Verify `host_permissions` in manifest.json

## Test Checklist

- [ ] Extension loads without errors
- [ ] Popup shows "Connected" status
- [ ] DID is generated and displayed
- [ ] Can generate proof of view
- [ ] Can verify credentials
- [ ] Ad slots trigger automatic proof generation
- [ ] Events are dispatched correctly

## Expected Results

When working correctly:
- Extension icon appears in toolbar
- Popup shows connection status and DID
- `window.trustSidecar` API is available on web pages
- Proofs are generated for ad slots
- Events are dispatched for proof generation

## More Info

See [TESTING.md](TESTING.md) for detailed testing instructions.



