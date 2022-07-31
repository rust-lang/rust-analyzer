# Kate Text Editor

Support for the language server protocol is built into Kate through the LSP plugin, which is included by default.
It is preconfigured to use rust-analyzer for Rust sources since Kate 21.12.

Earlier versions allow you to use rust-analyzer through a simple settings change.
In the LSP Client settings of Kate, copy the content of the third tab "default parameters" to the second tab "server configuration".
Then in the configuration replace:

```json
        "rust": {
            "command": ["rls"],
            "rootIndicationFileNames": ["Cargo.lock", "Cargo.toml"],
            "url": "https://github.com/rust-lang/rls",
            "highlightingModeRegex": "^Rust$"
        },
```

With

```json
        "rust": {
            "command": ["rust-analyzer"],
            "rootIndicationFileNames": ["Cargo.lock", "Cargo.toml"],
            "url": "https://github.com/rust-lang/rust-analyzer",
            "highlightingModeRegex": "^Rust$"
        },
```

Then click on apply, and restart the LSP server for your rust project.
