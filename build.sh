#!/bin/sh
set -e
set -x
. /usr/lib/emsdk/emsdk_env.sh
rustup run 1.38.0 cargo build --target=wasm32-unknown-emscripten "$@"
