At its core, rust-analyzer is a **library** for semantic analysis of
Rust code as it changes over time. This manual focuses on a specific
usage of the library — running it as part of a server that implements
the [Language Server
Protocol](https://microsoft.github.io/language-server-protocol/) (LSP).
The LSP allows various code editors, like VS Code, Emacs or Vim, to
implement semantic features like completion or goto definition by
talking to an external language server process.

<div class="tip">

To improve this document, send a pull request:  
[https://github.com/rust-analyzer/…​/manual.adoc](https://github.com/rust-lang/rust-analyzer/blob/master/docs/manual/src/README.md)

The manual is written in [AsciiDoc](https://asciidoc.org) and includes
some extra files which are generated from the source code. Run
`cargo test` and `cargo test -p xtask` to create these and then
`asciidoctor manual.adoc` to create an HTML copy.

</div>

If you have questions about using rust-analyzer, please ask them in the
[“IDEs and Editors”](https://users.rust-lang.org/c/ide/14) topic of Rust
users forum.
