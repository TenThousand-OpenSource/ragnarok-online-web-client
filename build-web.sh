#!/bin/sh
# Builds the browser client into client/dist (served by the gateway).
set -eu
cd "$(dirname "$0")"
cargo build --release -p roweb-client --target wasm32-unknown-unknown
mkdir -p client/dist
wasm-bindgen --target web --no-typescript --out-dir client/dist target/wasm32-unknown-unknown/release/roweb_client.wasm
cp client/web/index.html client/dist/
ls -lh client/dist/roweb_client_bg.wasm
