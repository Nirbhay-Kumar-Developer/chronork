#!/system/bin/sh
set -e

# --- Paths ---
PKG_NAME="chronork-aarch64"
STORAGE_PATH="/storage/emulated/0/Programming/chronork"
LOCAL_PATH="$HOME/chronork_tmp_build"

# Clean start
trap 'cp -r target "$STORAGE_PATH/" && rm -rf "$LOCAL_PATH"' EXIT 

# --- 0. Sync to Local (Faster I/O) ---
echo ">> Syncing to local storage..."
mkdir -p "$LOCAL_PATH"
cp -r "$STORAGE_PATH/." "$LOCAL_PATH/"
cd "$LOCAL_PATH"

# --- 2. Compile C++ Native Engine ---
echo ">> Running Makefile..."
cargo build

# --- 4. Package Assembly ---
echo ">> Assembling Debian Package..."
DEB_ROOT="$LOCAL_PATH/$PKG_NAME"
PREFIX_PATH="$DEB_ROOT/data/data/com.termux/files/usr"

# Create destinations
BIN_DEST="$PREFIX_PATH/bin"

mkdir -p "$BIN_DEST"

strip build/chronork
# A. Copy Primary Binary to /usr/bin
cp build/chronork "$BIN_DEST/"

# --- 5. Permissions & Build ---
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

# --- 6. Deployment ---
echo ">> Installing locally..."
dpkg --install chronork-aarch64.deb
# Copy the finished .deb back to your Programming folder
cp "${PKG_NAME}.deb" "$STORAGE_PATH/"
cp "target" "$STORAGE_PATH"

echo "🚀 Success!"
echo "Binary: $PREFIX/usr/bin/chronork"