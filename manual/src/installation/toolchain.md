# Toolchain

Only the latest stable standard library source is officially supported for use with rust-analyzer. If you are using an older toolchain or have an override set, rust-analyzer may fail to understand the Rust source. You will either need to update your toolchain or use an older version of rust-analyzer that is compatible with your toolchain.

If you are using an override in your project, you can still force rust-analyzer to use the stable toolchain via the environment variable RUSTUP_TOOLCHAIN. For example, with VS Code or coc-rust-analyzer:

```json
{ "rust-analyzer.server.extraEnv": { "RUSTUP_TOOLCHAIN": "stable" } }
```
