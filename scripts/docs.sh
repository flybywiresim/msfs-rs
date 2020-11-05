#!/bin/bash

set -ex

rm -rf docs
cargo doc --no-deps --workspace
mv target/doc docs
