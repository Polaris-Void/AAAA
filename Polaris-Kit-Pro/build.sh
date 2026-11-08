#!/usr/bin/env sh
set -eu

APP_NAME='Polaris-Kit-Pro'
VERSION='3.1.0'
PROJECT_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$PROJECT_ROOT"

echo "=============================================================="
echo "  $APP_NAME v$VERSION  |  release build (sh)"
echo "=============================================================="

if ! command -v cargo >/dev/null 2>&1; then
    echo '[ERROR] Rust/Cargo was not found. Install the stable Rust toolchain from rustup.rs.' >&2
    exit 1
fi

HOST_TARGET=$(rustc -vV | sed -n 's/^host: //p')
TARGET=${1:-$HOST_TARGET}

if [ "$TARGET" != "$HOST_TARGET" ]; then
    if ! command -v rustup >/dev/null 2>&1; then
        echo '[ERROR] rustup is required to install a non-host Rust target.' >&2
        exit 1
    fi
    rustup target add "$TARGET"
fi

cargo build --locked --release --target "$TARGET"

case "$TARGET" in
    *windows*)
        SOURCE="target/$TARGET/release/polaris-kit-pro.exe"
        DEST_DIR='dist'
        DEST="$DEST_DIR/$APP_NAME.exe"
        ;;
    *)
        SOURCE="target/$TARGET/release/polaris-kit-pro"
        DEST_DIR='dist'
        DEST="$DEST_DIR/$APP_NAME"
        ;;
esac

if [ ! -f "$SOURCE" ]; then
    echo "[ERROR] Build output not found: $SOURCE" >&2
    exit 1
fi
mkdir -p "$DEST_DIR"
cp -f "$SOURCE" "$DEST"
printf 'BUILD OK: %s\n' "$DEST"
