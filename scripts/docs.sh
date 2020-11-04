#!/bin/bash

rm -rf docs
cargo doc --no-deps --workspace
mv target/doc docs
