#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Install once: rustup target add wasm32-unknown-unknown
# Install the CLI matching Cargo.lock: cargo install wasm-bindgen-cli --version 0.2.130 --locked
BINDGEN="${PAINT_WASM_BINDGEN:-wasm-bindgen}"
if ! command -v "$BINDGEN" >/dev/null; then
  BINDGEN="$PWD/artifacts/tools/wasm-bindgen-0.2.130-x86_64-unknown-linux-musl/wasm-bindgen"
fi
[[ "$("$BINDGEN" --version)" == "wasm-bindgen 0.2.130" ]] || { echo 'wasm-bindgen CLI 0.2.130 is required' >&2; exit 1; }
cargo build -p openaec-paint-studio --lib --locked --release --target wasm32-unknown-unknown --features web
OUTPUT="${PAINT_WEB_OUT:-web-dist}"
mkdir -p "$OUTPUT/pkg"
"$BINDGEN" target/wasm32-unknown-unknown/release/openaec_paint_studio.wasm --target web --out-dir "$OUTPUT/pkg"
cp web/index.html "$OUTPUT/index.html"
printf '%s\n' 'Ready: python3 -m http.server 3020 --bind 127.0.0.1 --directory web-dist'
