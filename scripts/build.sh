#!/usr/bin/env bash
# Builds the client (Rust -> WebAssembly) and the server (Rust -> native), then stages the site.
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/env.sh
PROFILE_SERVER="${1:-release}"

echo "==> [1/4] Client: compiling Rust to WebAssembly (hydrate)"
cargo build --lib --target wasm32-unknown-unknown --profile wasm-release --no-default-features --features hydrate

echo "==> [2/4] Client: generating JS bindings (wasm-bindgen)"
mkdir -p target/site/pkg
wasm-bindgen target/wasm32-unknown-unknown/wasm-release/meridian_medical.wasm \
  --target web --no-typescript --remove-name-section --remove-producers-section --out-dir target/site/pkg --out-name meridian_medical

echo "==> [3/4] Static assets (CSS, fonts, icon)"
cp -r public/. target/site/
# Vercel serves public/ from its CDN, so the WASM bundle is published there too.
mkdir -p public/pkg && cp target/site/pkg/* public/pkg/

echo "==> [4/4] Server: compiling Axum + Leptos SSR"
cargo build --bin meridian_medical --features ssr --profile "$PROFILE_SERVER"
echo "Done. Run: target/$PROFILE_SERVER/meridian_medical.exe"
