#!/bin/bash
set -e

REPO="brunohaetinger/dir-nuke"
INSTALL_DIR="/usr/local/bin"
TMP_DIR=$(mktemp -d)
ARCH=$(uname -m)
OS=$(uname -s)

# Determine the correct target triple
case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
      aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
      *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      x86_64) TARGET="x86_64-apple-darwin" ;;
      arm64) TARGET="aarch64-apple-darwin" ;;
      *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac

echo "Detected platform: $TARGET"

# Get latest release tag using GitHub API
echo "Fetching latest version..."
LATEST=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [[ -z "$LATEST" ]]; then
  echo "Could not determine the latest version."
  exit 1
fi

echo "Latest version: $LATEST"

FILENAME="dir-nuke-${LATEST}-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${LATEST}/${FILENAME}"

echo "Downloading $URL..."
curl -L "$URL" -o "$TMP_DIR/$FILENAME"

echo "Extracting..."
tar -xzf "$TMP_DIR/$FILENAME" -C "$TMP_DIR"

echo "Installing to $INSTALL_DIR..."
chmod +x "$TMP_DIR/dir-nuke"
sudo mv "$TMP_DIR/dir-nuke" "$INSTALL_DIR/dir-nuke"

echo "✅ dir-nuke installed successfully!"
echo "Run with: dir-nuke --help"

# Clean up
rm -rf "$TMP_DIR"
