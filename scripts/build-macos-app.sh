#!/usr/bin/env bash
set -euo pipefail

# Build a releasable macOS .app bundle and DMG for kimi-usage-widget.
# Run from the repository root or via `./scripts/build-macos-app.sh`.

cd "$(dirname "$0")/.."

VERSION=$(grep '^version' Cargo.toml | head -n1 | sed -E 's/version *= *"([^"]+)"/\1/')
APP_NAME="Kimi Usage Widget"
BUNDLE_NAME="${APP_NAME}.app"
DIST_DIR="dist"
CONTENTS_DIR="${DIST_DIR}/${BUNDLE_NAME}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

echo "Building ${BUNDLE_NAME} v${VERSION}..."

cargo build --release

rm -rf "${DIST_DIR:?}/${BUNDLE_NAME}"
mkdir -p "${MACOS_DIR}" "${RESOURCES_DIR}"

cp "target/release/kimi-usage-widget" "${MACOS_DIR}/"

# Generate Info.plist with the current crate version.
sed "s/__VERSION__/${VERSION}/g" "packaging/macos/Info.plist" > "${CONTENTS_DIR}/Info.plist"

# Generate AppIcon.icns from assets/icon.png.
# The source icon is small, so we scale it to the sizes macOS expects.
TMP_DIR=$(mktemp -d)
ICONSET_DIR="${TMP_DIR}/AppIcon.iconset"
mkdir -p "${ICONSET_DIR}"
trap 'rm -rf "${TMP_DIR}"' EXIT

SIZES=(16 32 64 128 256 512)
for SIZE in "${SIZES[@]}"; do
    RETINA_SIZE=$((SIZE * 2))
    sips -z "${SIZE}" "${SIZE}" "assets/icon.png" \
        --out "${ICONSET_DIR}/icon_${SIZE}x${SIZE}.png" >/dev/null
    sips -z "${RETINA_SIZE}" "${RETINA_SIZE}" "assets/icon.png" \
        --out "${ICONSET_DIR}/icon_${SIZE}x${SIZE}@2x.png" >/dev/null
done
# 512x512@2x is the 1024x1024 required by iconutil.
sips -z 1024 1024 "assets/icon.png" \
    --out "${ICONSET_DIR}/icon_512x512@2x.png" >/dev/null

iconutil -c icns "${ICONSET_DIR}" -o "${RESOURCES_DIR}/AppIcon.icns"

# Ad-hoc sign the bundle so it launches from the DMG without being rejected.
codesign --force --deep --sign - "${DIST_DIR}/${BUNDLE_NAME}"

# Package the app into a compressed DMG for distribution.
DMG_NAME="Kimi-Usage-Widget-${VERSION}.dmg"
hdiutil create \
    -volname "${APP_NAME}" \
    -srcfolder "${DIST_DIR}/${BUNDLE_NAME}" \
    -ov \
    -format UDZO \
    "${DIST_DIR}/${DMG_NAME}" >/dev/null

echo "Created ${DIST_DIR}/${BUNDLE_NAME}"
echo "Created ${DIST_DIR}/${DMG_NAME}"
