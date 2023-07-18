#! /usr/bin/bash
cargo xtask dist
gunzip dist/rust-analyzer-x86_64-unknown-linux-gnu.gz
chmod +x dist/rust-analyzer-x86_64-unknown-linux-gnu

# cargo watch -- ./build_gunzip_chmod.sh