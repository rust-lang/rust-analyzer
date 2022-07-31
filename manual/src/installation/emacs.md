# Emacs

Note this excellent [guide](https://robert.kra.hn/posts/2021-02-07_rust-with-emacs/) from [@rksm](https://github.com/rksm).

Prerequisites: You have installed the [rust-analyzer binary](server_binary.md).

Emacs support is maintained as part of the [Emacs-LSP](https://github.com/emacs-lsp/lsp-mode) package in [lsp-rust.el](https://github.com/emacs-lsp/lsp-mode/blob/master/lsp-rust.el).

1. Install the most recent version of `emacs-lsp` package by following the [Emacs-LSP instructions](https://github.com/emacs-lsp/lsp-mode).
1. Set `lsp-rust-server` to `'rust-analyzer`.
1. Run `lsp` in a Rust buffer.
1. (Optionally) bind commands like `lsp-rust-analyzer-join-lines`, `lsp-extend-selection` and `lsp-rust-analyzer-expand-macro` to keys.
