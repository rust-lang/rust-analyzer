# VS Code

This is the best supported editor at the moment. The rust-analyzer plugin for VS Code is maintained in tree.

You can install the latest release of the plugin from the marketplace.

Note that the plugin may cause conflicts with the official Rust plugin. It is recommended to disable the Rust plugin when using the rust-analyzer extension.

By default, the plugin will prompt you to download the matching version of the server as well:

![VS Code Warning](/images/installation/vscode-1.png)

To disable this notification put the following in `settings.json`:

```json
{ "rust-analyzer.updates.askBeforeDownload": false }
```

The server binary is stored in the extension install directory, which starts with `rust-lang.rust-analyzer-` and is located under:

* Linux: `~/.vscode/extensions`
* Linux (Remote, such as WSL): `~/.vscode-server/extensions`
* macOS: `~/.vscode/extensions`
* Windows: `%USERPROFILE%\.vscode\extensions`

As an exception, on NixOS, the extension makes a copy of the server and stores it under `~/.config/Code/User/globalStorage/rust-lang.rust-analyzer`.

Note that we only support the two most recent versions of VS Code.

## Updates

The extension will be updated automatically as new versions become available.
It will ask your permission to download the matching language server version binary if needed.

### Nightly

We ship nightly releases for VS Code.
To help us out by testing the newest code, you can enable pre-release versions in the Code extension page.

## Manual Installation

Alternatively, download a VSIX corresponding to your platform from the
[releases](https://github.com/rust-lang/rust-analyzer/releases) page.

Install the extension with the `Extensions: Install from VSIX` command within VS Code, or from the command line via:

```shell
code --install-extension /path/to/rust-analyzer.vsix
```

If you are running an unsupported platform, you can install `rust-analyzer-no-server.vsix` and compile or obtain a server binary.
Copy the server anywhere, then add the path to your settings.json, for example:

```json
 "rust-analyzer.server.path": "~/.local/bin/rust-analyzer-linux" }
```

## Building From Source

Both the server and the Code plugin can be installed from source:

```shell
git clone https://github.com/rust-lang/rust-analyzer.git && cd rust-analyzer
cargo xtask install
```

You'll need Cargo, nodejs (matching a supported version of VS Code) and npm for this.

Note that installing via `xtask install` does not work for VS Code Remote, instead you'll need to install the `.vsix` manually.

If you're not using Code, you can compile and install only the LSP server:

```shell
cargo xtask install --server
```
