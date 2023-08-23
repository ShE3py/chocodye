#!/bin/bash

set -e
cd $(dirname -- $(readlink --canonicalize-existing -- $BASH_SOURCE))
cargo build --release
cp ./target/wasm32-unknown-unknown/release/chocoweb.wasm ./src/
