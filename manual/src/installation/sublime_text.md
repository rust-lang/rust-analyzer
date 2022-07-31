# Sublime Text

## Sublime Text 4

* Follow the instructions in [LSP-rust-analyzer](https://github.com/sublimelsp/LSP-rust-analyzer).

> NOTE: Install [LSP-file-watcher-chokidar](https://packagecontrol.io/packages/LSP-file-watcher-chokidar) to enable file watching (`workspace/didChangeWatchedFiles`).

## Sublime Text 3

* Install the [rust-analyzer binary](server_binary.md).
* Install the [LSP package](https://packagecontrol.io/packages/LSP).
* From the command palette, run `LSP: Enable Language Server Globally` and select `rust-analyzer`.

If it worked, you should see "rust-analyzer, Line X, Column Y" on the left side of the status bar, and after waiting a bit, functionalities like tooltips on hovering over variables should become available.

If you get an error saying `No such file or directory: 'rust-analyzer'`, see the [rust-analyzer binary](server_binary.md) section on installing the language server binary.
