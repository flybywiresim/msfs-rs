#!/bin/bash

rm -rf docs
cargo doc --no-deps --target wasm32-wasi
mv target/doc docs
