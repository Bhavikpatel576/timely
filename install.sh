#!/bin/sh
set -e

# Timely installer — installs Timely.app bundle to /Applications
# Usage: curl -fsSL <raw-url>/install.sh | sh

REPO="bhavikpatel/timely"
APP_DEST="/Applications/Timely.app"
SYMLINK_PATH="/usr/local/bin/timely"

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
  arm64|aarch64) ARCH="arm64" ;;
  x86_64) ARCH="x86_64" ;;
  *)
    echo "Error: Unsupported architecture: $ARCH"
    exit 1
    ;;
esac

OS=$(uname -s)
if [ "$OS" != "Darwin" ]; then
  echo "Error: Only macOS is supported at this time (got $OS)"
  exit 1
fi

echo "Detecting latest release..."
LATEST=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST" ]; then
  echo "Error: Could not determine latest release"
  exit 1
fi

TAR_NAME="timely-${LATEST}-${ARCH}-apple-darwin.tar.gz"
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST/$TAR_NAME"

echo "Downloading Timely $LATEST for $ARCH..."
TMPDIR=$(mktemp -d)
curl -fsSL "$DOWNLOAD_URL" -o "$TMPDIR/$TAR_NAME"

echo "Extracting..."
tar -xzf "$TMPDIR/$TAR_NAME" -C "$TMPDIR"

echo "Installing Timely.app to /Applications..."
if [ -d "$APP_DEST" ]; then
  rm -rf "$APP_DEST"
fi

if [ -w "/Applications" ]; then
  mv "$TMPDIR/Timely.app" "$APP_DEST"
else
  sudo mv "$TMPDIR/Timely.app" "$APP_DEST"
fi

echo "Creating CLI symlink at $SYMLINK_PATH..."
if [ -w "$(dirname "$SYMLINK_PATH")" ]; then
  ln -sf "$APP_DEST/Contents/MacOS/timely" "$SYMLINK_PATH"
else
  sudo ln -sf "$APP_DEST/Contents/MacOS/timely" "$SYMLINK_PATH"
fi

rm -rf "$TMPDIR"

echo ""
echo "Timely $LATEST installed successfully!"
echo "  App: $APP_DEST"
echo "  CLI: $SYMLINK_PATH"
echo ""
echo "Next steps:"
echo "  1. Open System Settings > Privacy & Security > Accessibility"
echo "  2. Click '+' and add /Applications/Timely.app"
echo "  3. Run: timely daemon start"
echo ""
echo "The daemon will start automatically on login."
