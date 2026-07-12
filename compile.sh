#!/system/bin/sh
set -e

# --- Paths ---
PKG_NAME="chronork-aarch64"
STORAGE_PATH="/storage/emulated/0/Programming/chronork"
LOCAL_PATH="$HOME/chronork_tmp_build"

# --- Arguments Parser ---
PROFILE="release"
BUILD_JNI=false

for arg in "$@"; do
    case "$arg" in
        debug)   PROFILE="debug" ;;
        release) PROFILE="release" ;;
        jni)     BUILD_JNI=true ;;
        *)       echo "Unknown argument: $arg"; exit 1 ;;
    esac
done

TARGET_BINARY="target/$PROFILE/chronork"
JNI_TARGET_DIR="target/aarch64-linux-android/$PROFILE"
JNI_BINARY="$JNI_TARGET_DIR/libchronork.so"

# Clean start (Copies target directory back to shared storage for incremental cache)
trap 'cp -r target "$STORAGE_PATH/" && rm -rf "$LOCAL_PATH"' EXIT 

# --- 1. Sync to Local (Faster I/O) ---
echo ">> Syncing to local storage..."
mkdir -p "$LOCAL_PATH"
cp -r "$STORAGE_PATH/." "$LOCAL_PATH/"
cd "$LOCAL_PATH"

# Maintain absolute execution bits on cached dependencies to prevent OS Error 13
if [ -d "target" ]; then
    echo ">> Restoring execution permissions to all cached build scripts..."
    find target -type f -path "*/build/*" -exec chmod +x {} + 2>/dev/null || true
fi

# --- 2. Setup Build Flags ---
CARGO_FLAGS=""
if [ "$PROFILE" = "release" ]; then
    CARGO_FLAGS="--release"
fi

# --- 3. Execution Branching ---
if [ "$BUILD_JNI" = true ]; then
    # --- JNI WORKFLOW ONLY ---
    echo ">> Running Cargo for Android JNI target..."
    cargo build $CARGO_FLAGS -p chronork-jni --target aarch64-linux-android

    if [ -f "$JNI_BINARY" ]; then
        mkdir -p "$STORAGE_PATH/lib/arm64-v8a"
        cp "$JNI_BINARY" "$STORAGE_PATH/lib/arm64-v8a/"
        echo ">> 🚀 JNI Success! Exported library -> $STORAGE_PATH/lib/arm64-v8a/libchronork.so"
    else
        echo ">> Error: JNI compilation reported success but library object was not found."
        exit 1
    fi
else
    # --- CLI WORKFLOW ONLY ---
    echo ">> Running Cargo for native CLI target..."
    cargo build $CARGO_FLAGS --bin chronork

    echo ">> Assembling Debian Package..."
    DEB_ROOT="$LOCAL_PATH/$PKG_NAME"
    PREFIX_PATH="$DEB_ROOT/data/data/com.termux/files/usr"

    BIN_DEST="$PREFIX_PATH/bin"
    mkdir -p "$BIN_DEST"
    mkdir -p "$STORAGE_PATH/build/$PROFILE"

    strip "$TARGET_BINARY"
    cp "$TARGET_BINARY" "$BIN_DEST/"

    echo ">> Setting Permissions..."
    find "$DEB_ROOT" -type d -exec chmod 755 {} +

    if [ -d "$DEB_ROOT/DEBIAN" ]; then
        find "$DEB_ROOT/DEBIAN" -type f -exec chmod 644 {} +
        [ -f "$DEB_ROOT/DEBIAN/postinst" ] && chmod 755 "$DEB_ROOT/DEBIAN/postinst"
    fi

    chmod +x "$BIN_DEST/chronork"

    echo ">> Building .deb..."
    dpkg-deb --build "$PKG_NAME"

    echo ">> Installing locally..."
    dpkg --install "${PKG_NAME}.deb"

    # Copy outputs back to persistent shared storage
    cp "${PKG_NAME}.deb" "$STORAGE_PATH/"
    cp "$TARGET_BINARY" "$STORAGE_PATH/build/$PROFILE/"
    
    echo "🚀 CLI Success!"
    echo "Binary: $PREFIX/bin/chronork"
fi