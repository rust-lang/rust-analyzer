#! /usr/bin/bash
cargo xtask dist --proof-action
gunzip dist/verus-analyzer-x86_64-unknown-linux-gnu.gz
chmod +x dist/verus-analyzer-x86_64-unknown-linux-gnu

# cargo watch -- ./build_gunzip_chmod.sh