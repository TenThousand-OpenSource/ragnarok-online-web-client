#!/bin/sh
# Builds the browser client into client/dist (served by the gateway).
set -eu
cd "$(dirname "$0")"
cargo build --release -p roweb-client --target wasm32-unknown-unknown
mkdir -p client/dist
wasm-bindgen --target web --no-typescript --out-dir client/dist target/wasm32-unknown-unknown/release/roweb_client.wasm
# Each build has its own number in the addresses of its files (see index.html).
sed "s/__BUILD__/$(date +%s)/g" client/web/index.html > client/dist/index.html
ls -lh client/dist/roweb_client_bg.wasm
