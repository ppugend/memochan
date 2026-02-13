#!/bin/bash
# Create DMG installer for macOS

set -e

APP_NAME="MemoChan"
VERSION="0.1.0"
BUNDLE_DIR="target/release/bundle/osx"
SOURCE_APP="${BUNDLE_DIR}/${APP_NAME}.app"
DMG_NAME="${APP_NAME}-${VERSION}.dmg"
TMP_DMG="tmp-${DMG_NAME}"
VOLUME_NAME="${APP_NAME}"

# Check if .app exists
if [ ! -d "$SOURCE_APP" ]; then
    echo "Building ${APP_NAME}.app..."
    cargo bundle --release
fi

echo "Creating DMG for ${APP_NAME}..."

# Remove old DMG files
rm -f "${BUNDLE_DIR}/${DMG_NAME}" "${BUNDLE_DIR}/${TMP_DMG}"

# Create temporary DMG
hdiutil create -srcfolder "$SOURCE_APP" -volname "$VOLUME_NAME" -fs HFS+ \
    -fsargs "-c c=64,a=16,e=16" -format UDRW "${BUNDLE_DIR}/${TMP_DMG}"

# Mount DMG and configure
DEVICE=$(hdiutil attach -readwrite -noverify "${BUNDLE_DIR}/${TMP_DMG}" | \
    egrep '^/dev/' | sed 1q | awk '{print $1}')

sleep 2

# Create Applications symlink
mkdir -p "/Volumes/${VOLUME_NAME}"
ln -s /Applications "/Volumes/${VOLUME_NAME}/Applications"

# Set background and icon position (optional)
mkdir -p "/Volumes/${VOLUME_NAME}/.background"

# Unmount
hdiutil detach "${DEVICE}"

# Convert to final DMG
hdiutil convert "${BUNDLE_DIR}/${TMP_DMG}" -format UDZO \
    -imagekey zlib-level=9 -o "${BUNDLE_DIR}/${DMG_NAME}"

# Cleanup
rm -f "${BUNDLE_DIR}/${TMP_DMG}"

echo "DMG created: ${BUNDLE_DIR}/${DMG_NAME}"
