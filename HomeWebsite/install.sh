#!/usr/bin/env bash
set -euo pipefail

# ── Config ────────────────────────────────────────────────────────────────────
# Basic identity
APP_NAME="ScanCodeSync"           # Display name (used for osascript, pgrep, log messages)
APP_BUNDLE="ScanCodeSync.app"     # Actual .app bundle name inside the archive / DMG
BUNDLE_ID="com.ollielynas.scancodesync"  # macOS bundle identifier

# Update endpoint
UPDATE_URL="https://sync-home.ollielynas.com/latest.json"
PLATFORM_KEY="macos"              # Key inside json["platforms"] to use

# JSON field paths (python3 expressions against the parsed dict `d`)
JSON_VERSION_EXPR="d['version']"
JSON_URL_EXPR="d['platforms']['${PLATFORM_KEY}']['url']"
JSON_SHA256_EXPR="d['platforms']['${PLATFORM_KEY}']['signature'].lstrip('sha256:')"

# Install location
APPDIR="/Applications"

# Zap paths removed on --uninstall (space-separated, ~ is expanded automatically)
ZAP_PATHS=(
  "$HOME/Library/Application Support/$APP_NAME"
  "$HOME/Library/Preferences/$BUNDLE_ID.plist"
  "$HOME/Library/Logs/$APP_NAME"
)

TMP_DIR="$(mktemp -d)"

# ── Cleanup on exit ───────────────────────────────────────────────────────────
cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

# ── Fetch latest metadata ─────────────────────────────────────────────────────
echo "Fetching latest version info..."
JSON="$(curl -fsSL "$UPDATE_URL")"

VERSION="$(echo "$JSON" | python3 -c "import sys,json; d=json.load(sys.stdin); print($JSON_VERSION_EXPR)")"
URL="$(    echo "$JSON" | python3 -c "import sys,json; d=json.load(sys.stdin); print($JSON_URL_EXPR)")"
SHA256="$(  echo "$JSON" | python3 -c "import sys,json; d=json.load(sys.stdin); print($JSON_SHA256_EXPR)")"

echo "  Version : $VERSION"
echo "  URL     : $URL"
echo "  SHA-256 : $SHA256"

# ── Download ──────────────────────────────────────────────────────────────────
FILENAME="$(basename "$URL")"
DOWNLOAD_PATH="$TMP_DIR/$FILENAME"

echo "Downloading $APP_NAME $VERSION..."
curl -fL --progress-bar -o "$DOWNLOAD_PATH" "$URL"

# ── Verify checksum ───────────────────────────────────────────────────────────
echo "Verifying checksum..."
ACTUAL_SHA="$(shasum -a 256 "$DOWNLOAD_PATH" | awk '{print $1}')"
if [ "$ACTUAL_SHA" != "$SHA256" ]; then
  echo "ERROR: Checksum mismatch!"
  echo "  Expected : $SHA256"
  echo "  Actual   : $ACTUAL_SHA"
  exit 1
fi
echo "  Checksum OK"

# ── Quit running instance (if any) ───────────────────────────────────────────
if pgrep -x "$APP_NAME" > /dev/null 2>&1; then
  echo "Quitting running instance of $APP_NAME..."
  osascript -e "quit app \"$APP_NAME\"" 2>/dev/null || true
  sleep 1
fi

# ── Install ───────────────────────────────────────────────────────────────────
echo "Installing $APP_NAME..."
case "$FILENAME" in
  *.dmg)
    MOUNT_POINT="$(hdiutil attach -nobrowse -quiet "$DOWNLOAD_PATH" \
      | awk 'END{print $NF}')"
    cp -R "$MOUNT_POINT/$APP_BUNDLE" "$APPDIR/"
    hdiutil detach "$MOUNT_POINT" -quiet
    ;;
  *.zip)
    unzip -q "$DOWNLOAD_PATH" -d "$TMP_DIR"
    cp -R "$TMP_DIR/$APP_BUNDLE" "$APPDIR/"
    ;;
  *.tar.gz|*.tgz)
    tar -xzf "$DOWNLOAD_PATH" -C "$TMP_DIR"
    cp -R "$TMP_DIR/$APP_BUNDLE" "$APPDIR/"
    ;;
  *)
    echo "ERROR: Unknown archive format: $FILENAME"
    exit 1
    ;;
esac

# ── Remove quarantine ─────────────────────────────────────────────────────────
echo "Removing quarantine attribute..."
/usr/bin/xattr -dr com.apple.quarantine "$APPDIR/$APP_BUNDLE"

echo ""
echo "$APP_NAME $VERSION installed successfully to $APPDIR/$APP_BUNDLE"

# ── Uninstall helper (run with --uninstall) ───────────────────────────────────
if [[ "${1:-}" == "--uninstall" ]]; then
  echo "Uninstalling $APP_NAME..."

  # Quit the app
  osascript -e "quit app \"$APP_NAME\"" 2>/dev/null || true
  sleep 1

  # Remove app bundle
  rm -rf "$APPDIR/$APP_NAME.app"

  # Zap leftover files
  rm -rf \
    "$HOME/Library/Application Support/$APP_NAME" \
    "$HOME/Library/Preferences/$BUNDLE_ID.plist" \
    "$HOME/Library/Logs/$APP_NAME"

  echo "$APP_NAME has been uninstalled."
  exit 0
fi
