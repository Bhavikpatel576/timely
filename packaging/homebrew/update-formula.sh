#!/bin/bash
set -euo pipefail

# Updates the Homebrew formula with a new version and SHA256 checksums.
# Usage: ./update-formula.sh <version> <arm64-sha256> <x86_64-sha256> [formula-path]
#
# Called by CI after a release is published.

VERSION="${1:?Usage: update-formula.sh <version> <arm64-sha256> <x86_64-sha256> [formula-path]}"
ARM64_SHA="${2:?Missing arm64 SHA256}"
X86_64_SHA="${3:?Missing x86_64 SHA256}"
FORMULA_PATH="${4:-Formula/timely.rb}"

if [ ! -f "$FORMULA_PATH" ]; then
  echo "Error: Formula not found at $FORMULA_PATH"
  exit 1
fi

echo "Updating formula at $FORMULA_PATH"
echo "  Version:    $VERSION"
echo "  ARM64 SHA:  $ARM64_SHA"
echo "  x86_64 SHA: $X86_64_SHA"

# Update version
sed -i '' "s/version \".*\"/version \"$VERSION\"/" "$FORMULA_PATH"

# Update SHA256 checksums — arm64 is the first sha256 line, x86_64 is the second
# Use awk to replace them in order
awk -v arm="$ARM64_SHA" -v x86="$X86_64_SHA" '
  /sha256/ && !done_arm { sub(/sha256 ".*"/, "sha256 \"" arm "\""); done_arm=1; next }
  /sha256/ && done_arm && !done_x86 { sub(/sha256 ".*"/, "sha256 \"" x86 "\""); done_x86=1 }
  { print }
' "$FORMULA_PATH" > "$FORMULA_PATH.tmp" && mv "$FORMULA_PATH.tmp" "$FORMULA_PATH"

echo "Formula updated successfully."
