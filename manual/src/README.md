# The Official Rust Analyzer Book

At its core, rust-analyzer is a *library* for semantic analysis of Rust code as it changes over time.
This manual focuses on a specific usage of the library -- running it as part of a server that implements the
[Language Server Protocol (LSP)](https://microsoft.github.io/language-server-protocol/).
The LSP allows various code editors, like VS Code, Emacs or Vim, to implement semantic features like completion or goto definition by talking to an external language server process.

> To improve this document, please open a pull request: <https://github.com/rust-lang/rust-analyzer/tree/master/manual>. The manual is written in [Markdown](https://commonmark.org/help/) using [mdbook](https://rust-lang.github.io/mdBook/).

If you have questions about using rust-analyzer, please ask them in the [Editors and IDEs](https://users.rust-lang.org/c/ide/14) topic of the [Rust users forum](https://users.rust-lang.org/).
