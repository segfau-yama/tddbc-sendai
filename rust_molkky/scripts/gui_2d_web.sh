#!/bin/sh
set -eu
cd "$(dirname "$0")/.."

command -v wasm-bindgen >/dev/null || {
    echo "READMEの手順でwasm-bindgen-cliを導入してください。" >&2
    exit 1
}
cargo build --locked --no-default-features --features gui_2d_web --bin gui_2d --target wasm32-unknown-unknown
wasm-bindgen --target web --out-name gui_2d --out-dir target/gui_2d_web target/wasm32-unknown-unknown/debug/gui_2d.wasm
cp web/index.html target/gui_2d_web/index.html
exec python3 -m http.server "${PORT:-8080}" --bind 127.0.0.1 --directory target/gui_2d_web
