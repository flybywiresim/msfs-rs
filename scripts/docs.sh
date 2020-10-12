#!/bin/bash

rm -rf docs
cargo doc --no-deps
mv target/doc docs
