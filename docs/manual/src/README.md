
![rust-analyzer](https://rust-analyzer.github.io/assets/rust-analyzer.svg)

At its core, rust-analyzer is a **library** for semantic analysis of
Rust code as it changes over time. This manual focuses on a specific
usage of the library — running it as part of a server that implements
the [Language Server
Protocol](https://microsoft.github.io/language-server-protocol/) (LSP).
The LSP allows various code editors, like VS Code, Emacs or Vim, to
implement semantic features like completion or goto definition by
talking to an external language server process.

This manual is written in [Markdown](https://commonmark.org/help/) and created using
[mdbook](https://github.com/rust-lang/mdBook). Some content is generated
from source. Please see the [Documentation](contributing/documentation.md)
section of the [Contributing](contributing/README.md) guide for more information.

If you have questions about using rust-analyzer, please ask them in the
[“IDEs and Editors”](https://users.rust-lang.org/c/ide/14) topic of Rust
users forum.
