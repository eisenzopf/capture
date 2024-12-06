#!/bin/bash
set -e

# Get the certificate name from our development keychain
CERT_NAME=$(security find-identity -v -p codesigning Development.keychain | grep -o '".*"' | sed 's/"//g')
echo "Using certificate: $CERT_NAME"

# Build the app
cargo clean
cargo build --release
cargo bundle --release

# Set up paths
APP_NAME="Audio Capture"
BUILD_DIR="target/release/bundle/osx"
APP_PATH="$BUILD_DIR/$APP_NAME.app"
DMG_PATH="target/AudioCapture.dmg"

# Unlock the keychain (replace 'temppass' with the password you used)
security unlock-keychain -p temppass Development.keychain

# Sign the app with entitlements
echo "Signing application..."
codesign --force --options runtime \
    --entitlements "entitlements.plist" \
    --keychain Development.keychain \
    --sign "$CERT_NAME" \
    "$APP_PATH"

# Verify signing
echo "Verifying signature..."
codesign -dvv "$APP_PATH"

# Create DMG
echo "Creating DMG..."
create-dmg \
    --volname "Audio Capture" \
    --window-pos 200 120 \
    --window-size 800 400 \
    --icon-size 100 \
    --icon "$APP_NAME.app" 200 190 \
    --hide-extension "$APP_NAME.app" \
    --app-drop-link 600 185 \
    "$DMG_PATH" \
    "$APP_PATH"

# Sign the DMG
echo "Signing DMG..."
codesign --force \
    --keychain Development.keychain \
    --sign "$CERT_NAME" \
    "$DMG_PATH"

echo "Done! DMG created at $DMG_PATH"