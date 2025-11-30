#!/bin/bash
set -e

REPO="a-lebailly/hyprspace"
BINARY_NAME="hyprspace"

echo "Retrieving latest release information..."
LATEST_API_URL="https://api.github.com/repos/$REPO/releases/latest"
DOWNLOAD_URL=$(curl -sSL "$LATEST_API_URL" | grep "browser_download_url" | grep "$BINARY_NAME" | cut -d '"' -f 4)

if [ -z "$DOWNLOAD_URL" ]; then
    echo "Error: no '$BINARY_NAME' binary found in the latest release."
    echo "Ensure that a compiled '$BINARY_NAME' binary is attached to the most recent GitHub release."
    exit 1
fi

echo "Downloading '$BINARY_NAME' from:"
echo "  $DOWNLOAD_URL"
echo ""

curl -sSL "$DOWNLOAD_URL" -o "$BINARY_NAME"
chmod +x "$BINARY_NAME"

echo ""
echo "Download complete."
echo "The binary has been saved in the current directory as:"
echo "  ./$BINARY_NAME"
echo ""

echo "To install the binary system-wide, run:"
echo "  sudo mv ./$BINARY_NAME /usr/local/bin/$BINARY_NAME"
echo ""

echo "Available commands:"
echo "  hyprspace   Launch the TUI"