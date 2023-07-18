### using cargo watch
Developing verus-analyzer can be time-consuming when it needs to interact with VS Code. In that case, we can use `cargo watch` to automatically build and run the analyzer. 

```bash
#! /usr/bin/bash
cargo xtask dist
gunzip dist/rust-analyzer-x86_64-unknown-linux-gnu.gz
chmod +x dist/rust-analyzer-x86_64-unknown-linux-gnu
```

In a separate tmux pane, run this with `cargo watch -- ./build_gunzip_chmod.sh`

When it finishes, do `Developer: Reload Window` in VS Code to reload verus-analyzer.
