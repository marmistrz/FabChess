#!/bin/sh
set -e
set -x
. /usr/lib/emsdk/emsdk_env.sh
export RUSTFLAGS="-C link-arg=-s -C link-arg=ALLOW_MEMORY_GROWTH=1"
rustup run 1.38.0 cargo build --target=wasm32-unknown-emscripten "$@"
