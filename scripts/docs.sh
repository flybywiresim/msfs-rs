#!/bin/bash

rm -rf docs
cargo doc
mv target/wasm32-wasi/doc docs
