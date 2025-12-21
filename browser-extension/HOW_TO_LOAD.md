# How to Load the Extension - Step by Step

## The Problem: Files Are Grayed Out

When files appear grayed out in the "Load unpacked" dialog, it usually means:
- You're selecting the wrong directory
- Chrome can't find `manifest.json` in the selected directory
- You need to select the **`browser-extension`** folder specifically

## Correct Way to Load

### Step 1: Open Extensions Page
1. Open Chrome/Edge
2. Go to `chrome://extensions/` (or `edge://extensions/`)
3. Enable **"Developer mode"** (toggle in top-right corner)

### Step 2: Click "Load unpacked"
- Click the **"Load unpacked"** button (top-left)

### Step 3: Navigate to the CORRECT Directory
**IMPORTANT**: You must select the `browser-extension` folder, NOT the parent directory!

**Correct path:**
```
/path/to/trust-sidecar/browser-extension
```

**What to do:**
1. In the file picker, navigate to the repository root directory
2. **Click on the `browser-extension` folder** (don't go inside it yet)
3. **Click "Select"** or "Open"

**OR** if you're already inside `browser-extension`:
- Make sure you can see `manifest.json` in the file list
- If you see it, click "Select" on the current directory
- If files are grayed out, go up one level and then select `browser-extension`

### Step 4: Verify It Loaded
After selecting, you should see:
- "Trust Sidecar" appears in the extensions list
- No red error messages
- Extension icon appears in your toolbar

## Common Mistakes

### Mistake 1: Selecting Parent Directory
**Wrong:**
```
/path/to/trust-sidecar  ← DON'T select this
```

**Right:**
```
/path/to/trust-sidecar/browser-extension  ← Select THIS
```

### Mistake 2: Going Inside browser-extension
- Don't double-click into `browser-extension` and then try to select files
- Select the `browser-extension` folder itself

### Mistake 3: Wrong Path
Make sure you're in the right location. The `manifest.json` must be directly in the folder you select.

## How to Verify You're in the Right Place

Before clicking "Select", check:
1. You should see `manifest.json` in the file list
2. You should see `background.js`, `content.js`, `popup.html`
3. You should see an `icons` folder

If you see these files, you're in the right place!

## If It Still Doesn't Work

### Check 1: File Permissions
```bash
cd browser-extension
ls -la manifest.json
# Should show: -rw-rw-r-- (readable)
```

### Check 2: Manifest is Valid
```bash
cd browser-extension
cat manifest.json | head -3
# Should show: { "manifest_version": 3, ...
```

### Check 3: Try Absolute Path
In the file picker, you can paste the absolute path to the `browser-extension` directory

### Check 4: Try Different Browser
- Try Edge instead of Chrome (or vice versa)
- Some browsers handle file pickers differently

## Quick Checklist

- [ ] Developer mode is enabled
- [ ] Clicked "Load unpacked"
- [ ] Navigated to the repository root directory
- [ ] Selected the `browser-extension` folder (not parent, not inside it)
- [ ] Can see `manifest.json` in the file list before selecting
- [ ] Clicked "Select" or "Open"
- [ ] Extension appears in the list without errors

## Visual Guide

```
trust-sidecar/                    ← Don't select this
├── src/
├── Cargo.toml
└── browser-extension/            ← SELECT THIS FOLDER
    ├── manifest.json            ← This file must be visible
    ├── background.js
    ├── content.js
    ├── popup.html
    └── icons/
```

When you select `browser-extension`, Chrome should immediately recognize it as an extension because `manifest.json` is in the root of that folder.
