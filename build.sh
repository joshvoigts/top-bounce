#!/bin/bash
set -e

APP_NAME="TopBounce"
RUST_CRATE_NAME="top-bounce"
TARGET_DIR="$(cargo metadata | jq -r ".target_directory")"
RELEASE_DIR="$TARGET_DIR/release"
APP_DIR="$RELEASE_DIR/${APP_NAME}.app"

echo "Target directory: $TARGET_DIR"

# Build release binary
echo "Building release binary..."
cargo build --release

# Create the app bundle
echo "Creating app bundle..."
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

# Copy Info.plist and icon
cp Info.plist "$APP_DIR/Contents/Info.plist"

# Copy the binary
cp "$RELEASE_DIR/${RUST_CRATE_NAME}" "$APP_DIR/Contents/MacOS/"

# Remove quarantine attribute (bypass Gatekeeper for local use)
xattr -cr "$APP_DIR"

# Ad-hoc sign the app
codesign --force --sign - --deep "$APP_DIR"

echo "✅ Built ${APP_NAME}.app"
echo ""
echo "To install:"
echo "  1. Copy $APP_DIR to /Applications/"
echo "  2. Run the app - it will prompt for Accessibility permissions"
echo "  3. Grant access in System Settings > Privacy & Security > Accessibility"
echo ""
echo "The app will now run without needing terminal accessibility access!"

# Also show where the app is
echo ""
echo "App location: $APP_DIR"
