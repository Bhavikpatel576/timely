#!/bin/sh
set -e

# Assemble Timely.app macOS bundle
# Usage: bundle.sh <binary-path> <version> <output-dir>

BINARY="$1"
VERSION="$2"
OUTPUT_DIR="$3"

if [ -z "$BINARY" ] || [ -z "$VERSION" ] || [ -z "$OUTPUT_DIR" ]; then
    echo "Usage: bundle.sh <binary-path> <version> <output-dir>"
    echo "  binary-path  Path to the compiled timely binary"
    echo "  version      Version string (e.g. 0.1.0)"
    echo "  output-dir   Directory to create Timely.app in"
    exit 1
fi

if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found at $BINARY"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_DIR="$OUTPUT_DIR/Timely.app"
CONTENTS="$APP_DIR/Contents"

echo "Creating Timely.app (v$VERSION)..."

# Clean previous bundle
rm -rf "$APP_DIR"

# Create directory structure
mkdir -p "$CONTENTS/MacOS"
mkdir -p "$CONTENTS/Resources"

# Copy binary
cp "$BINARY" "$CONTENTS/MacOS/timely"
chmod +x "$CONTENTS/MacOS/timely"

# Generate Info.plist with version substituted
sed "s/VERSION_PLACEHOLDER/$VERSION/g" "$SCRIPT_DIR/Info.plist" > "$CONTENTS/Info.plist"

# Ad-hoc code sign
codesign --force --sign - "$APP_DIR"

echo "Created $APP_DIR"
echo "  Binary:  $CONTENTS/MacOS/timely"
echo "  Version: $VERSION"

# Verify
codesign --verify "$APP_DIR" && echo "Code signature: valid" || echo "Warning: code signature verification failed"
