#!/usr/bin/env bash
set -euo pipefail

# ── Config ────────────────────────────────────────────────────────────────────
# Basic identity
APP_NAME="ScanCodeSync"                      # Display name (used for osascript, pgrep, log messages)
APP_BUNDLE="ScanCodeSync.app"                # Actual .app bundle name inside the archive / DMG
BUNDLE_ID="com.ollielynas.scancodesync"      # macOS bundle identifier

# Update endpoint
UPDATE_URL="https://sync-home.ollielynas.com/latest.json"
PLATFORM_KEY="macos"                         # Key inside json["platforms"] to use

# JSON field paths (python3 expressions against the parsed dict `d`)
JSON_VERSION_EXPR="d['version']"
JSON_URL_EXPR="d['platforms']['${PLATFORM_KEY}']['url']"
# NOTE: use replace(..., 1) not lstrip() — lstrip strips individual chars, not a prefix
JSON_SHA256_EXPR="d['platforms']['${PLATFORM_KEY}']['signature'].replace('sha256:', '', 1)"

# Install location
APPDIR="/Applications"

# Zap paths removed on --uninstall
ZAP_PATHS=(
  "$HOME/Library/Application Support/$APP_NAME"
  "$HOME/Library/Preferences/$BUNDLE_ID.plist"
  "$HOME/Library/Logs/$APP_NAME"
)

# ── Uninstall (must be checked before any install logic runs) ─────────────────
if [[ "${1:-}" == "--uninstall" ]]; then
  echo "Uninstalling $APP_NAME..."

  osascript -e "quit app \"$APP_NAME\"" 2>/dev/null || true
  sleep 1

  rm -rf "$APPDIR/$APP_BUNDLE"

  for zap_path in "${ZAP_PATHS[@]}"; do
    rm -rf "$zap_path"
  done

  echo "$APP_NAME has been uninstalled."
  exit 0
fi

# ── Temp dir + cleanup ────────────────────────────────────────────────────────
TMP_DIR="$(mktemp -d)"
MOUNT_POINT=""

cleanup() {
  # Detach any DMG that was mounted by this script
  if [ -n "$MOUNT_POINT" ] && [ -d "$MOUNT_POINT" ]; then
    hdiutil detach "$MOUNT_POINT" -quiet 2>/dev/null || true
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

# ── Fetch latest metadata ─────────────────────────────────────────────────────
echo "Fetching latest version info..."
JSON="$(curl -fsSL "$UPDATE_URL")"

VERSION="$(echo "$JSON" | python3 -c "import sys,json; d=json.load(sys.stdin); print($JSON_VERSION_EXPR)")"
URL="$(    echo "$JSON" | python3 -c "import sys,json; d=json.load(sys.stdin); print($JSON_URL_EXPR)")"
SHA256="$(  echo "$JSON" | python3 -c "import sys,json; d=json.load(sys.stdin); print($JSON_SHA256_EXPR)")"

if [ -z "$VERSION" ] || [ -z "$URL" ] || [ -z "$SHA256" ]; then
  echo "ERROR: Could not parse version metadata from $UPDATE_URL"
  exit 1
fi

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
  sleep 2
fi

# ── Install ───────────────────────────────────────────────────────────────────
echo "Installing $APP_NAME..."

# Helper: find the .app in a directory (up to 2 levels deep)
find_app() {
  local search_dir="$1"
  find "$search_dir" -maxdepth 2 -name "$APP_BUNDLE" -type d | head -1
}

case "$FILENAME" in
  *.dmg)
    # Match the line containing the /Volumes/ mount path — works for both
    # HFS+ and APFS DMGs regardless of partition type string
    MOUNT_POINT="$(hdiutil attach -nobrowse "$DOWNLOAD_PATH" \
      | grep '/Volumes/' | awk '{print substr($0, index($0,"/Volumes/"))}' || true)"
    if [ -z "$MOUNT_POINT" ]; then
      echo "ERROR: Failed to determine DMG mount point"
      exit 1
    fi
    echo "  Mounted at: $MOUNT_POINT"

    APP_SRC="$(find_app "$MOUNT_POINT")"
    if [ -z "$APP_SRC" ]; then
      echo "ERROR: Could not find $APP_BUNDLE in mounted DMG"
      exit 1
    fi

    # Remove old version before copying to avoid stale file merges
    rm -rf "$APPDIR/$APP_BUNDLE"
    cp -R "$APP_SRC" "$APPDIR/"

    hdiutil detach "$MOUNT_POINT" -quiet
    MOUNT_POINT=""   # Mark as detached so cleanup trap skips it
    ;;

  *.zip)
    unzip -q "$DOWNLOAD_PATH" -d "$TMP_DIR/extracted"
    APP_SRC="$(find_app "$TMP_DIR/extracted")"
    if [ -z "$APP_SRC" ]; then
      echo "ERROR: Could not find $APP_BUNDLE in zip archive"
      exit 1
    fi
    rm -rf "$APPDIR/$APP_BUNDLE"
    cp -R "$APP_SRC" "$APPDIR/"
    ;;

  *.tar.gz|*.tgz)
    mkdir -p "$TMP_DIR/extracted"
    tar -xzf "$DOWNLOAD_PATH" -C "$TMP_DIR/extracted"
    APP_SRC="$(find_app "$TMP_DIR/extracted")"
    if [ -z "$APP_SRC" ]; then
      echo "ERROR: Could not find $APP_BUNDLE in tar archive"
      exit 1
    fi
    rm -rf "$APPDIR/$APP_BUNDLE"
    cp -R "$APP_SRC" "$APPDIR/"
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
