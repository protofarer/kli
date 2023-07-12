#!/bin/bash

OUT_DIR="$HOME/bin"

cargo build --release

cp target/release/kli $OUT_DIR
echo "Build done, binary moved to $OUT_DIR"
