#!/bin/bash
# Update Homebrew formula with new version and SHA256
# Usage: ./scripts/update-formula.sh <version> <tarball-url>

set -e

VERSION="${1:-1.0.0-rc.1}"
TARBALL_URL="${2:-https://github.com/yourusername/pip-rs/archive/refs/tags/v${VERSION}.tar.gz}"

echo "Downloading tarball to calculate SHA256..."
SHA256=$(curl -sL "$TARBALL_URL" | shasum -a 256 | cut -d' ' -f1)

if [ "$SHA256" = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855" ]; then
    echo "Error: Empty file downloaded. Check the URL."
    exit 1
fi

echo "Version: $VERSION"
echo "SHA256: $SHA256"

# Update formula
sed -i.bak "s|url \".*\"|url \"$TARBALL_URL\"|" Formula/pip-rs.rb
sed -i.bak "s|sha256 \".*\"|sha256 \"$SHA256\"|" Formula/pip-rs.rb
rm -f Formula/pip-rs.rb.bak

echo "Formula updated!"
