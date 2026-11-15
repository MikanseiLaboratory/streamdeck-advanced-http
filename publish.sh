#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
PLUGIN_ID="dev.flowingspdg.advancedhttp"
PLUGIN_DIR="$ROOT/plugin/$PLUGIN_ID.sdPlugin"
OUTPUT_DIR="${OUTPUT_DIR:-$ROOT/artifacts/plugin}"
STAGE="${STAGE:-$ROOT/artifacts/binaries}"
INSTALL="${INSTALL:-0}"
PACK="${PACK:-0}"

publish_target() {
  local target="$1"
  local name="$2"
  rustup target add "$target"
  cargo build -p advanced-http-plugin --release --bin plugin --target "$target"
  mkdir -p "$STAGE/$target"
  local src="$ROOT/target/$target/release/plugin"
  if [[ "$name" == *.exe ]]; then
    src="${src}.exe"
  fi
  cp "$src" "$STAGE/$target/$name"
}

cd "$ROOT"
cargo run -p advanced-http-plugin --bin typegen
(cd "$ROOT/pi" && { [ -d node_modules ] || npm install; } && npm run build)

publish_target x86_64-pc-windows-msvc plugin.exe || true
publish_target aarch64-apple-darwin plugin || true
publish_target x86_64-apple-darwin plugin || true

mkdir -p "$PLUGIN_DIR/bin"
WIN="$STAGE/x86_64-pc-windows-msvc/plugin.exe"
ARM="$STAGE/aarch64-apple-darwin/plugin"
X64="$STAGE/x86_64-apple-darwin/plugin"
if [[ -f "$WIN" ]]; then
  cp "$WIN" "$PLUGIN_DIR/bin/plugin.exe"
fi
if [[ -f "$ARM" && -f "$X64" ]]; then
  lipo -create "$ARM" "$X64" -output "$PLUGIN_DIR/bin/plugin"
elif [[ -f "$ARM" ]]; then
  cp "$ARM" "$PLUGIN_DIR/bin/plugin"
elif [[ -f "$X64" ]]; then
  cp "$X64" "$PLUGIN_DIR/bin/plugin"
fi
if [[ -f "$PLUGIN_DIR/bin/plugin" ]]; then
  chmod +x "$PLUGIN_DIR/bin/plugin"
fi

if [[ "$PACK" == "1" ]]; then
  mkdir -p "$OUTPUT_DIR"
  ZIP="$OUTPUT_DIR/$PLUGIN_ID.streamDeckPlugin"
  rm -f "$ZIP"
  if command -v streamdeck >/dev/null 2>&1; then
    streamdeck pack "$PLUGIN_DIR" --output "$OUTPUT_DIR" --force
  else
    (cd "$(dirname "$PLUGIN_DIR")" && zip -qr "$ZIP" "$(basename "$PLUGIN_DIR")")
  fi
  echo "Packed $ZIP"
fi

if [[ "$INSTALL" == "1" ]]; then
  DEST="$HOME/Library/Application Support/com.elgato.StreamDeck/Plugins/$PLUGIN_ID.sdPlugin"
  mkdir -p "$(dirname "$DEST")"
  rm -rf "$DEST"
  cp -R "$PLUGIN_DIR" "$DEST"
  echo "Installed plugin to $DEST"
  if command -v streamdeck >/dev/null 2>&1; then
    streamdeck restart "$PLUGIN_ID"
  fi
fi
