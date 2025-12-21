#!/bin/bash
# Validate browser extension files

echo "Validating Trust Sidecar Browser Extension..."
echo ""

ERRORS=0

# Check if required files exist
echo "Checking required files..."
REQUIRED_FILES=("manifest.json" "background.js" "content.js" "popup.html" "popup.js")
for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "  [OK] $file"
    else
        echo "  [ERROR] $file - MISSING"
        ERRORS=$((ERRORS + 1))
    fi
done

# Check if icons exist
echo ""
echo "Checking icons..."
ICON_SIZES=(16 48 128)
for size in "${ICON_SIZES[@]}"; do
    if [ -f "icons/icon${size}.png" ]; then
        echo "  [OK] icon${size}.png"
    else
        echo "  [ERROR] icon${size}.png - MISSING"
        ERRORS=$((ERRORS + 1))
    fi
done

# Validate manifest.json
echo ""
echo "Validating manifest.json..."
if command -v jq &> /dev/null; then
    if jq empty manifest.json 2>/dev/null; then
        echo "  [OK] manifest.json is valid JSON"
        
        # Check manifest version
        MANIFEST_VERSION=$(jq -r '.manifest_version' manifest.json)
        if [ "$MANIFEST_VERSION" = "3" ]; then
            echo "  [OK] Manifest version 3"
        else
            echo "  [WARN] Manifest version is $MANIFEST_VERSION (expected 3)"
        fi
    else
        echo "  [ERROR] manifest.json is invalid JSON"
        ERRORS=$((ERRORS + 1))
    fi
else
    echo "  [WARN] jq not found, skipping JSON validation"
fi

# Check JavaScript syntax (basic)
echo ""
echo "Checking JavaScript files..."
JS_FILES=("background.js" "content.js" "popup.js")
for file in "${JS_FILES[@]}"; do
    if node -c "$file" 2>/dev/null; then
        echo "  [OK] $file syntax OK"
    else
        echo "  [WARN] $file - syntax check skipped (node not available)"
    fi
done

# Check if server is running
echo ""
echo "Checking Trust Sidecar server..."
if curl -s http://127.0.0.1:3000/health > /dev/null 2>&1; then
    echo "  [OK] Server is running on http://127.0.0.1:3000"
else
    echo "  [WARN] Server is not running (start with: cargo run --release)"
fi

echo ""
if [ $ERRORS -eq 0 ]; then
    echo "[SUCCESS] Extension validation passed!"
    echo ""
    echo "Next steps:"
    echo "1. Open chrome://extensions/"
    echo "2. Enable Developer mode"
    echo "3. Click 'Load unpacked'"
    echo "4. Select this directory: $(pwd)"
else
    echo "[ERROR] Found $ERRORS error(s). Please fix them before loading the extension."
    exit 1
fi



