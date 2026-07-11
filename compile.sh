#!/system/bin/sh
set -e

# --- Paths ---
PKG_NAME="chronork-aarch64"
STORAGE_PATH="/storage/emulated/0/Programming/chronork"
LOCAL_PATH="$HOME/chronork_tmp_build"

# Default to release if no argument is passed
PROFILE="${1:-release}"
TARGET_BINARY="target/$PROFILE/chronork"
JNI_TARGET_DIR="target/aarch64-linux-android/$PROFILE"
JNI_BINARY="$JNI_TARGET_DIR/libchronork_jni.so"

# Clean start (Copies target directory back to shared storage for incremental cache)
trap 'cp -r target "$STORAGE_PATH/" && rm -rf "$LOCAL_PATH"' EXIT 

# --- 1. Sync to Local (Faster I/O) ---
echo ">> Syncing to local storage..."
mkdir -p "$LOCAL_PATH"
cp -r "$STORAGE_PATH/." "$LOCAL_PATH/"
cd "$LOCAL_PATH"

# --- 2. Compile Rust (Workspace Setup) ---
echo ">> Running Cargo..."
if [ "$PROFILE" = "release" ]; then
    # Target the CLI specifically for the Debian package
    cargo build --release --bin chronork
    
    # Optional: Build the Android JNI library simultaneously
    # Uncomment the next line once your NDK is configured in Termux
    # cargo build --release -p chronork_jni --target aarch64-linux-android
else
    cargo build --bin chronork
    # cargo build -p chronork_jni --target aarch64-linux-android
fi

# --- 3. Package Assembly ---
echo ">> Assembling Debian Package..."
DEB_ROOT="$LOCAL_PATH/$PKG_NAME"
PREFIX_PATH="$DEB_ROOT/data/data/com.termux/files/usr"

# Create destinations
BIN_DEST="$PREFIX_PATH/bin"
mkdir -p "$BIN_DEST"
mkdir -p "$STORAGE_PATH/build/$PROFILE"

# A. Strip and Copy Primary Binary to /usr/bin
strip "$TARGET_BINARY"
cp "$TARGET_BINARY" "$BIN_DEST/"

# --- 4. Permissions & Build ---
echo ">> Setting Permissions..."
find "$DEB_ROOT" -type d -exec chmod 755 {} +

# Avoid chmoding DEBIAN folder if it doesn't exist in the local build
if [ -d "$DEB_ROOT/DEBIAN" ]; then
    find "$DEB_ROOT/DEBIAN" -type f -exec chmod 644 {} +
    [ -f "$DEB_ROOT/DEBIAN/postinst" ] && chmod 755 "$DEB_ROOT/DEBIAN/postinst"
fi

chmod +x "$BIN_DEST/chronork"

echo ">> Building .deb..."
dpkg-deb --build "$PKG_NAME"

# --- 5. Deployment ---
echo ">> Installing locally..."
dpkg --install "${PKG_NAME}.deb"

# Copy outputs back to your Programming folder
cp "${PKG_NAME}.deb" "$STORAGE_PATH/"
cp "$TARGET_BINARY" "$STORAGE_PATH/build/$PROFILE/"

# Export JNI library if it was built
if [ -f "$JNI_BINARY" ]; then
    cp "$JNI_BINARY" "$STORAGE_PATH/build/$PROFILE/"
    echo ">> Exported JNI library to build directory."
fi

echo "🚀 Success!"
echo "Binary: $PREFIX/bin/chronork"