#!/bin/bash
# Generate update manifest for Tauri updater

VERSION=$1
PLATFORM=$2
FILE_PATH=$3
SIGNATURE_FILE="${FILE_PATH}.sig"

# Read signature if it exists
SIGNATURE=""
if [ -f "$SIGNATURE_FILE" ]; then
  SIGNATURE=$(cat "$SIGNATURE_FILE")
fi

# Get file URL
FILENAME=$(basename "$FILE_PATH")
URL="https://shenghsi.github.io/pomohardo-releases/${FILENAME}"

# Generate JSON
cat << EOF
{
  "version": "$VERSION",
  "pub_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "platforms": {
    "$PLATFORM": {
      "url": "$URL",
      "signature": "$SIGNATURE"
    }
  }
}
EOF
