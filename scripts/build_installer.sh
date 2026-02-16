#!/bin/bash
set -e

TARGET="${1:-x86_64-pc-windows-gnu}"

case "$TARGET" in
    x86_64-pc-windows-gnu) ARCH="amd64" ;;
    aarch64-pc-windows-msvc) ARCH="arm64" ;;
    *) ARCH="$TARGET" ;;
esac

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

if [ ! -f "$PROJECT_ROOT/Cargo.toml" ]; then
    echo "Error: Cargo.toml not found. Please run this script from the project root."
    exit 1
fi

cd "$PROJECT_ROOT"

VERSION=$(grep '^version =' Cargo.toml | head -n 1 | cut -d '"' -f 2)
APP_NAME="MemoChan"
PUBLISHER="ppugend"

OUTPUT_DIR="$PROJECT_ROOT/target/${TARGET}/release/bundle"
EXE_PATH="$PROJECT_ROOT/target/${TARGET}/release/memochan.exe"
EXE_NAME="memochan.exe"
LICENSE_PATH="$PROJECT_ROOT/LICENSE"

echo "=========================================="
echo "Building ${APP_NAME} v${VERSION} for ${TARGET}"
echo "=========================================="

echo ""
echo "[1/2] Building cross-compiled binary..."
if [ "$TARGET" = "aarch64-pc-windows-msvc" ]; then
    cargo xwin build --release --target "$TARGET"
else
    cargo build --release --target "$TARGET"
fi

echo ""
echo "[2/2] Creating NSIS installer..."

mkdir -p "$OUTPUT_DIR"

makensis \
  -DAPP_NAME="${APP_NAME}" \
  -DVERSION="${VERSION}" \
  -DPUBLISHER="${PUBLISHER}" \
  -DARCH="${ARCH}" \
  -DEXE_NAME="${EXE_NAME}" \
  -DEXE_PATH="${EXE_PATH}" \
  -DOUTPUT_DIR="${OUTPUT_DIR}" \
  -DLICENSE_PATH="${LICENSE_PATH}" \
  "$SCRIPT_DIR/installer.nsi"

echo ""
echo "=========================================="
echo "Done! Installer: ${OUTPUT_DIR}/${APP_NAME}_Setup_${VERSION}_${ARCH}.exe"
echo "=========================================="
