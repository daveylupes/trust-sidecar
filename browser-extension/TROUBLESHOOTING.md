# Troubleshooting Browser Extension

This guide helps resolve common issues with the Trust Sidecar browser extension.

## Issue: "Extension not detected"

### Symptoms
- Test page shows " Trust Sidecar extension not detected"
- `window.trustSidecar` is undefined in console

### Solutions

#### 1. Verify Extension is Loaded
1. Open `chrome://extensions/` (or `edge://extensions/`)
2. Find "Trust Sidecar" in the list
3. Make sure it's **enabled** (toggle is ON)
4. Check for any error messages (red text)

#### 2. Reload the Extension
1. In `chrome://extensions/`, find Trust Sidecar
2. Click the **reload icon** (circular arrow)
3. Refresh the test page

#### 3. Check Content Script Injection
1. Open the test page
2. Press F12 to open Developer Tools
3. Go to **Console** tab
4. Look for: `"Trust Sidecar content script loaded"`
5. Look for: `"Trust Sidecar API injected into page context"`

If you don't see these messages:
- The content script isn't running
- Check for errors in the console
- Verify `manifest.json` has correct content script configuration

#### 4. Check Background Script
1. In `chrome://extensions/`, find Trust Sidecar
2. Click **"Service worker"** or **"background page"** link
3. Check the console for errors
4. Look for: `"Trust Sidecar extension installed"`

#### 5. Verify Server Connection
1. Open browser console (F12)
2. Check for CORS errors
3. Test server manually:
   ```bash
   curl http://127.0.0.1:3000/health
   ```

#### 6. Check Permissions
1. In `chrome://extensions/`, find Trust Sidecar
2. Click **"Details"**
3. Verify permissions include:
   - Storage
   - Active Tab
   - Host permissions for `http://127.0.0.1:3000/*`

#### 7. Manual API Check
In the browser console (F12), try:
```javascript
// Wait a moment for injection
setTimeout(() => {
  console.log('trustSidecar available:', typeof window.trustSidecar);
  if (window.trustSidecar) {
    window.trustSidecar.generateProof('test').then(console.log).catch(console.error);
  }
}, 2000);
```

#### 8. Check for Conflicts
- Disable other extensions temporarily
- Try in an incognito window
- Clear browser cache

### Debug Steps

1. **Check Console Logs**
   - Open F12 → Console
   - Look for errors or warnings
   - Check for "Trust Sidecar" messages

2. **Inspect Content Script**
   - F12 → Sources tab
   - Look for `content.js` in the file tree
   - Set breakpoints to debug

3. **Check Network Requests**
   - F12 → Network tab
   - Filter by "127.0.0.1"
   - Verify requests to Trust Sidecar API

4. **Verify Manifest**
   - Check `manifest.json` is valid JSON
   - Verify `content_scripts` section exists
   - Check `matches` includes your test page URL

### Common Issues

#### Issue: Content script not running
**Solution**: 
- Check `manifest.json` `content_scripts.matches` includes your page
- Verify `run_at` is set correctly
- Reload extension and page

#### Issue: API not injected
**Solution**:
- Content script injection happens on page load
- Refresh the page after loading extension
- Check console for injection errors

#### Issue: CORS errors
**Solution**:
- Verify server is running
- Check `host_permissions` in manifest
- Server should allow CORS from browser

#### Issue: Background script not responding
**Solution**:
- Check service worker is running
- Look for errors in service worker console
- Reload extension

## Issue: Extension Loading Problems

### Files Grayed Out When Loading

If files appear grayed out in the "Load unpacked" dialog:

1. **Select the correct directory**: You must select the `browser-extension` folder, not the parent directory
2. **Verify manifest.json exists**: The file should be visible in the file list
3. **Check file permissions**: Ensure files are readable
4. **Try absolute path**: Paste the full path to the `browser-extension` directory

### Extension Won't Load

1. **Check manifest.json**: Run `./validate-extension.sh` to verify it's valid
2. **Check for errors**: Look for red error messages in `chrome://extensions/`
3. **Verify all files present**: Ensure all required files are in the directory
4. **Check browser compatibility**: Chrome 88+ or Edge 88+ (Manifest V3 required)

## Issue: Extension Needs Reload After Changes

After modifying extension files:

1. **Reload the extension**: Click the reload icon in `chrome://extensions/`
2. **Refresh the page**: Press Ctrl+R (or Cmd+R on Mac)
3. **For manifest.json changes**: Remove and re-add the extension

## Still Not Working?

1. **Check Extension Version**: Verify you have the latest files
2. **Reinstall Extension**: Remove and reload from `browser-extension` directory
3. **Check Browser Compatibility**: Chrome 88+ or Edge 88+ (Manifest V3)
4. **File Permissions**: Ensure all files are readable

### Getting Help

If issues persist:
1. Check browser console (F12) for errors
2. Check extension service worker console
3. Verify Trust Sidecar server is running: `curl http://127.0.0.1:3000/health`
4. Review [GETTING_STARTED.md](../GETTING_STARTED.md) for setup instructions



