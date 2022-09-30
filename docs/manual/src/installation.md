# Installation

In theory, one should be able to just install the [`rust-analyzer`
binary](#rust-analyzer-language-server-binary) and have it automatically
work with any editor. We are not there yet, so some editor specific
setup is required.

Additionally, rust-analyzer needs the sources of the standard library.
If the source code is not present, rust-analyzer will attempt to install
it automatically.

To add the sources manually, run the following command:

``` bash
$ rustup component add rust-src
```

<!-- toc -->

## Toolchain

Only the latest stable standard library source is officially supported
for use with rust-analyzer. If you are using an older toolchain or have
an override set, rust-analyzer may fail to understand the Rust source.
You will either need to update your toolchain or use an older version of
rust-analyzer that is compatible with your toolchain.

If you are using an override in your project, you can still force
rust-analyzer to use the stable toolchain via the environment variable
`RUSTUP_TOOLCHAIN`. For example, with VS Code or coc-rust-analyzer:

``` json
{ "rust-analyzer.server.extraEnv": { "RUSTUP_TOOLCHAIN": "stable" } }
```

## VS Code

This is the best supported editor at the moment. The rust-analyzer
plugin for VS Code is maintained [in
tree](https://github.com/rust-lang/rust-analyzer/tree/master/editors/code).

You can install the latest release of the plugin from [the
marketplace](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

Note that the plugin may cause conflicts with the [official Rust
plugin](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust).
It is recommended to disable the Rust plugin when using the
rust-analyzer extension.

By default, the plugin will prompt you to download the matching version
of the server as well:

![75067008 17502500 54ba 11ea 835a
f92aac50e866](https://user-images.githubusercontent.com/9021944/75067008-17502500-54ba-11ea-835a-f92aac50e866.png)

<div class="note">

To disable this notification put the following to `settings.json`

``` json
{ "rust-analyzer.updates.askBeforeDownload": false }
```

</div>

The server binary is stored in the extension install directory, which
starts with `rust-lang.rust-analyzer-` and is located under:

- Linux: `~/.vscode/extensions`

- Linux (Remote, such as WSL): `~/.vscode-server/extensions`

- macOS: `~/.vscode/extensions`

- Windows: `%USERPROFILE%\.vscode\extensions`

As an exception, on NixOS, the extension makes a copy of the server and
stores it under
`~/.config/Code/User/globalStorage/rust-lang.rust-analyzer`.

Note that we only support the two most recent versions of VS Code.

### Updates

The extension will be updated automatically as new versions become
available. It will ask your permission to download the matching language
server version binary if needed.

#### Nightly

We ship nightly releases for VS Code. To help us out by testing the
newest code, you can enable pre-release versions in the Code extension
page.

### Manual installation

Alternatively, download a VSIX corresponding to your platform from the
[releases](https://github.com/rust-lang/rust-analyzer/releases) page.

Install the extension with the `Extensions: Install from VSIX` command
within VS Code, or from the command line via:

    $ code --install-extension /path/to/rust-analyzer.vsix

If you are running an unsupported platform, you can install
`rust-analyzer-no-server.vsix` and compile or obtain a server binary.
Copy the server anywhere, then add the path to your settings.json, for
example:

``` json
{ "rust-analyzer.server.path": "~/.local/bin/rust-analyzer-linux" }
```

### Building From Source

Both the server and the Code plugin can be installed from source:

    $ git clone https://github.com/rust-lang/rust-analyzer.git && cd rust-analyzer
    $ cargo xtask install

You’ll need Cargo, nodejs (matching a supported version of VS Code) and
npm for this.

Note that installing via `xtask install` does not work for VS Code
Remote, instead you’ll need to install the `.vsix` manually.

If you’re not using Code, you can compile and install only the LSP
server:

    $ cargo xtask install --server

## rust-analyzer Language Server Binary

Other editors generally require the `rust-analyzer` binary to be in
`$PATH`. You can download pre-built binaries from the
[releases](https://github.com/rust-lang/rust-analyzer/releases) page.
You will need to uncompress and rename the binary for your platform,
e.g. from `rust-analyzer-aarch64-apple-darwin.gz` on Mac OS to
`rust-analyzer`, make it executable, then move it into a directory in
your `$PATH`.

On Linux to install the `rust-analyzer` binary into `~/.local/bin`,
these commands should work:

``` bash
$ mkdir -p ~/.local/bin
$ curl -L https://github.com/rust-lang/rust-analyzer/releases/latest/download/rust-analyzer-x86_64-unknown-linux-gnu.gz | gunzip -c - > ~/.local/bin/rust-analyzer
$ chmod +x ~/.local/bin/rust-analyzer
```

Make sure that `~/.local/bin` is listed in the `$PATH` variable and use
the appropriate URL if you’re not on a `x86-64` system.

You don’t have to use `~/.local/bin`, any other path like `~/.cargo/bin`
or `/usr/local/bin` will work just as well.

Alternatively, you can install it from source using the command below.
You’ll need the latest stable version of the Rust toolchain.

``` bash
$ git clone https://github.com/rust-lang/rust-analyzer.git && cd rust-analyzer
$ cargo xtask install --server
```

If your editor can’t find the binary even though the binary is on your
`$PATH`, the likely explanation is that it doesn’t see the same `$PATH`
as the shell, see [this
issue](https://github.com/rust-lang/rust-analyzer/issues/1811). On Unix,
running the editor from a shell or changing the `.desktop` file to set
the environment should help.

### `rustup`

`rust-analyzer` is available in `rustup`, but only in the nightly
toolchain:

``` bash
$ rustup +nightly component add rust-analyzer-preview
```

However, in contrast to `component add clippy` or
`component add rustfmt`, this does not actually place a `rust-analyzer`
binary in `~/.cargo/bin`, see [this
issue](https://github.com/rust-lang/rustup/issues/2411).

### Arch Linux

The `rust-analyzer` binary can be installed from the repos or AUR (Arch
User Repository):

- [`rust-analyzer`](https://www.archlinux.org/packages/community/x86_64/rust-analyzer/)
  (built from latest tagged source)

- [`rust-analyzer-git`](https://aur.archlinux.org/packages/rust-analyzer-git)
  (latest Git version)

Install it with pacman, for example:

``` bash
$ pacman -S rust-analyzer
```

### Gentoo Linux

`rust-analyzer` is available in the GURU repository:

- [`dev-util/rust-analyzer`](https://gitweb.gentoo.org/repo/proj/guru.git/tree/dev-util/rust-analyzer?id=9895cea62602cfe599bd48e0fb02127411ca6e81)
  builds from source

- [`dev-util/rust-analyzer-bin`](https://gitweb.gentoo.org/repo/proj/guru.git/tree/dev-util/rust-analyzer-bin?id=9895cea62602cfe599bd48e0fb02127411ca6e81)
  installs an official binary release

If not already, GURU must be enabled (e.g. using
`app-eselect/eselect-repository`) and sync’d before running `emerge`:

``` bash
$ eselect repository enable guru && emaint sync -r guru
$ emerge rust-analyzer-bin
```

### macOS

The `rust-analyzer` binary can be installed via
[Homebrew](https://brew.sh/).

``` bash
$ brew install rust-analyzer
```

## Emacs

Note this excellent
[guide](https://robert.kra.hn/posts/2021-02-07_rust-with-emacs/) from
[@rksm](https://github.com/rksm).

Prerequisites: You have installed the [`rust-analyzer`
binary](#rust-analyzer-language-server-binary).

Emacs support is maintained as part of the
[Emacs-LSP](https://github.com/emacs-lsp/lsp-mode) package in
[lsp-rust.el](https://github.com/emacs-lsp/lsp-mode/blob/master/lsp-rust.el).

1.  Install the most recent version of `emacs-lsp` package by following
    the [Emacs-LSP instructions](https://github.com/emacs-lsp/lsp-mode).

2.  Set `lsp-rust-server` to `'rust-analyzer`.

3.  Run `lsp` in a Rust buffer.

4.  (Optionally) bind commands like `lsp-rust-analyzer-join-lines`,
    `lsp-extend-selection` and `lsp-rust-analyzer-expand-macro` to keys.

## Vim/NeoVim

Prerequisites: You have installed the [`rust-analyzer`
binary](#rust-analyzer-language-server-binary). Not needed if the
extension can install/update it on its own, coc-rust-analyzer is one
example.

There are several LSP client implementations for vim or neovim:

### coc-rust-analyzer

1.  Install coc.nvim by following the instructions at
    [coc.nvim](https://github.com/neoclide/coc.nvim) (Node.js required)

2.  Run `:CocInstall coc-rust-analyzer` to install
    [coc-rust-analyzer](https://github.com/fannheyward/coc-rust-analyzer),
    this extension implements *most* of the features supported in the
    VSCode extension:

    - automatically install and upgrade stable/nightly releases

    - same configurations as VSCode extension,
      `rust-analyzer.server.path`, `rust-analyzer.cargo.features` etc.

    - same commands too, `rust-analyzer.analyzerStatus`,
      `rust-analyzer.ssr` etc.

    - inlay hints for variables and method chaining, *Neovim Only*

Note: for code actions, use `coc-codeaction-cursor` and
`coc-codeaction-selected`; `coc-codeaction` and `coc-codeaction-line`
are unlikely to be useful.

### LanguageClient-neovim

1.  Install LanguageClient-neovim by following the instructions
    [here](https://github.com/autozimu/LanguageClient-neovim)

    - The GitHub project wiki has extra tips on configuration

2.  Configure by adding this to your vim/neovim config file (replacing
    the existing Rust-specific line if it exists):

    ``` vim
    let g:LanguageClient_serverCommands = {
    \ 'rust': ['rust-analyzer'],
    \ }
    ```

### YouCompleteMe

Install YouCompleteMe by following the instructions
[here](https://github.com/ycm-core/YouCompleteMe#installation).

rust-analyzer is the default in ycm, it should work out of the box.

### ALE

To use the LSP server in [ale](https://github.com/dense-analysis/ale):

``` vim
let g:ale_linters = {'rust': ['analyzer']}
```

### nvim-lsp

NeoVim 0.5 has built-in language server support. For a quick start
configuration of rust-analyzer, use
[neovim/nvim-lspconfig](https://github.com/neovim/nvim-lspconfig#rust_analyzer).
Once `neovim/nvim-lspconfig` is installed, use
`lua require'lspconfig'.rust_analyzer.setup({})` in your `init.vim`.

You can also pass LSP settings to the server:

``` vim
lua << EOF
local nvim_lsp = require'lspconfig'

local on_attach = function(client)
    require'completion'.on_attach(client)
end

nvim_lsp.rust_analyzer.setup({
    on_attach=on_attach,
    settings = {
        ["rust-analyzer"] = {
            imports = {
                granularity = {
                    group = "module",
                },
                prefix = "self",
            },
            cargo = {
                buildScripts = {
                    enable = true,
                },
            },
            procMacro = {
                enable = true
            },
        }
    }
})
EOF
```

See <https://sharksforarms.dev/posts/neovim-rust/> for more tips on
getting started.

Check out <https://github.com/simrat39/rust-tools.nvim> for a batteries
included rust-analyzer setup for neovim.

### vim-lsp

vim-lsp is installed by following [the plugin
instructions](https://github.com/prabirshrestha/vim-lsp). It can be as
simple as adding this line to your `.vimrc`:

``` vim
Plug 'prabirshrestha/vim-lsp'
```

Next you need to register the `rust-analyzer` binary. If it is available
in `$PATH`, you may want to add this to your `.vimrc`:

``` vim
if executable('rust-analyzer')
  au User lsp_setup call lsp#register_server({
        \   'name': 'Rust Language Server',
        \   'cmd': {server_info->['rust-analyzer']},
        \   'whitelist': ['rust'],
        \ })
endif
```

There is no dedicated UI for the server configuration, so you would need
to send any options as a value of the `initialization_options` field, as
described in the [Configuration](#_configuration) section. Here is an
example of how to enable the proc-macro support:

``` vim
if executable('rust-analyzer')
  au User lsp_setup call lsp#register_server({
        \   'name': 'Rust Language Server',
        \   'cmd': {server_info->['rust-analyzer']},
        \   'whitelist': ['rust'],
        \   'initialization_options': {
        \     'cargo': {
        \       'buildScripts': {
        \         'enable': v:true,
        \       },
        \     },
        \     'procMacro': {
        \       'enable': v:true,
        \     },
        \   },
        \ })
endif
```

## Sublime Text

### Sublime Text 4:

- Follow the instructions in
  [LSP-rust-analyzer](https://github.com/sublimelsp/LSP-rust-analyzer).

<div class="note">

Install
[LSP-file-watcher-chokidar](https://packagecontrol.io/packages/LSP-file-watcher-chokidar)
to enable file watching (`workspace/didChangeWatchedFiles`).

</div>

### Sublime Text 3:

- Install the [`rust-analyzer`
  binary](#rust-analyzer-language-server-binary).

- Install the [LSP package](https://packagecontrol.io/packages/LSP).

- From the command palette, run `LSP: Enable Language Server Globally`
  and select `rust-analyzer`.

If it worked, you should see "rust-analyzer, Line X, Column Y" on the
left side of the status bar, and after waiting a bit, functionalities
like tooltips on hovering over variables should become available.

If you get an error saying `No such file or directory: 'rust-analyzer'`,
see the [`rust-analyzer` binary](#rust-analyzer-language-server-binary)
section on installing the language server binary.

## GNOME Builder

GNOME Builder 3.37.1 and newer has native `rust-analyzer` support. If
the LSP binary is not available, GNOME Builder can install it when
opening a Rust file.

## Eclipse IDE

Support for Rust development in the Eclipse IDE is provided by [Eclipse
Corrosion](https://github.com/eclipse/corrosion). If available in PATH
or in some standard location, `rust-analyzer` is detected and powers
editing of Rust files without further configuration. If `rust-analyzer`
is not detected, Corrosion will prompt you for configuration of your
Rust toolchain and language server with a link to the *Window \>
Preferences \> Rust* preference page; from here a button allows to
download and configure `rust-analyzer`, but you can also reference
another installation. You’ll need to close and reopen all .rs and Cargo
files, or to restart the IDE, for this change to take effect.

## Kate Text Editor

Support for the language server protocol is built into Kate through the
LSP plugin, which is included by default. It is preconfigured to use
rust-analyzer for Rust sources since Kate 21.12.

Earlier versions allow you to use rust-analyzer through a simple
settings change. In the LSP Client settings of Kate, copy the content of
the third tab "default parameters" to the second tab "server
configuration". Then in the configuration replace:

``` json
        "rust": {
            "command": ["rls"],
            "rootIndicationFileNames": ["Cargo.lock", "Cargo.toml"],
            "url": "https://github.com/rust-lang/rls",
            "highlightingModeRegex": "^Rust$"
        },
```

With

``` json
        "rust": {
            "command": ["rust-analyzer"],
            "rootIndicationFileNames": ["Cargo.lock", "Cargo.toml"],
            "url": "https://github.com/rust-lang/rust-analyzer",
            "highlightingModeRegex": "^Rust$"
        },
```

Then click on apply, and restart the LSP server for your rust project.

## juCi++

[juCi++](https://gitlab.com/cppit/jucipp) has built-in support for the
language server protocol, and since version 1.7.0 offers installation of
both Rust and rust-analyzer when opening a Rust file.

## Kakoune

[Kakoune](https://kakoune.org/) supports LSP with the help of
[`kak-lsp`](https://github.com/kak-lsp/kak-lsp). Follow the
[instructions](https://github.com/kak-lsp/kak-lsp#installation) to
install `kak-lsp`. To configure `kak-lsp`, refer to the [configuration
section](https://github.com/kak-lsp/kak-lsp#configuring-kak-lsp) which
is basically about copying the [configuration
file](https://github.com/kak-lsp/kak-lsp/blob/master/kak-lsp.toml) in
the right place (latest versions should use `rust-analyzer` by default).

Finally, you need to configure Kakoune to talk to `kak-lsp` (see [Usage
section](https://github.com/kak-lsp/kak-lsp#usage)). A basic
configuration will only get you LSP but you can also activate inlay
diagnostics and auto-formatting on save. The following might help you
get all of this.

``` txt
eval %sh{kak-lsp --kakoune -s $kak_session}  # Not needed if you load it with plug.kak.
hook global WinSetOption filetype=rust %{
    # Enable LSP
    lsp-enable-window

    # Auto-formatting on save
    hook window BufWritePre .* lsp-formatting-sync

    # Configure inlay hints (only on save)
    hook window -group rust-inlay-hints BufWritePost .* rust-analyzer-inlay-hints
    hook -once -always window WinSetOption filetype=.* %{
        remove-hooks window rust-inlay-hints
    }
}
```

## Helix

[Helix](https://docs.helix-editor.com/) supports LSP by default.
However, it won’t install `rust-analyzer` automatically. You can follow
instructions for installing [`rust-analyzer`
binary](#rust-analyzer-language-server-binary).