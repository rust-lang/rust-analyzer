# Eclipse IDE

Support for Rust development in the Eclipse IDE is provided by [Eclipse Corrosion](https://github.com/eclipse/corrosion).
If available in PATH or in some standard location, `rust-analyzer` is detected and powers editing of Rust files without further configuration.
If `rust-analyzer` is not detected, Corrosion will prompt you for configuration of your Rust toolchain and language server with a link to the __Window > Preferences > Rust__ preference page; from here a button allows to download and configure `rust-analyzer`, but you can also reference another installation.
You'll need to close and reopen all .rs and Cargo files, or to restart the IDE, for this change to take effect.
