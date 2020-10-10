#!/bin/bash

rm -rf docs
cargo doc
mv target/doc docs
