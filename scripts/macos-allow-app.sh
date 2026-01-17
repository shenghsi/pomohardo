#!/bin/bash
# Helper script to allow macOS app to run by removing quarantine attribute
# Usage: ./scripts/macos-allow-app.sh /path/to/Pomohardo.app

if [ -z "$1" ]; then
    echo "Usage: $0 /path/to/Pomohardo.app"
    echo "Example: $0 ~/Downloads/Pomohardo.app"
    exit 1
fi

APP_PATH="$1"

if [ ! -d "$APP_PATH" ]; then
    echo "Error: App not found at $APP_PATH"
    exit 1
fi

echo "Removing quarantine attribute from $APP_PATH..."
xattr -dr com.apple.quarantine "$APP_PATH"

echo ""
echo "Quarantine removed. Now:"
echo "1. Go to System Settings → Privacy & Security"
echo "2. Scroll to the Security section at the bottom"
echo "3. Click 'Open Anyway' next to the Pomohardo message"
echo "4. Confirm by clicking 'Open'"
