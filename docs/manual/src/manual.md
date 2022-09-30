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
[https://github.com/rust-analyzer/…​/manual.adoc](https://github.com/rust-lang/rust-analyzer/blob/master/docs/user/manual.adoc)

The manual is written in [AsciiDoc](https://asciidoc.org) and includes
some extra files which are generated from the source code. Run
`cargo test` and `cargo test -p xtask` to create these and then
`asciidoctor manual.adoc` to create an HTML copy.

</div>

If you have questions about using rust-analyzer, please ask them in the
[“IDEs and Editors”](https://users.rust-lang.org/c/ide/14) topic of Rust
users forum.

<!-- toc -->

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

# Troubleshooting

Start with looking at the rust-analyzer version. Try **rust-analyzer:
Show RA Version** in VS Code (using **Command Palette** feature
typically activated by Ctrl+Shift+P) or `rust-analyzer --version` in the
command line. If the date is more than a week ago, it’s better to update
rust-analyzer version.

The next thing to check would be panic messages in rust-analyzer’s log.
Log messages are printed to stderr, in VS Code you can see then in the
`Output > Rust Analyzer Language Server` tab of the panel. To see more
logs, set the `RA_LOG=info` environment variable, this can be done
either by setting the environment variable manually or by using
`rust-analyzer.server.extraEnv`, note that both of these approaches
require the server to be restarted.

To fully capture LSP messages between the editor and the server, set
`"rust-analyzer.trace.server": "verbose"` config and check
`Output > Rust Analyzer Language Server Trace`.

The root cause for many “nothing works” problems is that rust-analyzer
fails to understand the project structure. To debug that, first note the
`rust-analyzer` section in the status bar. If it has an error icon and
red, that’s the problem (hover will have somewhat helpful error
message). **rust-analyzer: Status** prints dependency information for
the current file. Finally, `RA_LOG=project_model=debug` enables verbose
logs during project loading.

If rust-analyzer outright crashes, try running
`rust-analyzer analysis-stats /path/to/project/directory/` on the
command line. This command type checks the whole project in batch mode
bypassing LSP machinery.

When filing issues, it is useful (but not necessary) to try to minimize
examples. An ideal bug reproduction looks like this:

``` bash
$ git clone https://github.com/username/repo.git && cd repo && git switch --detach commit-hash
$ rust-analyzer --version
rust-analyzer dd12184e4 2021-05-08 dev
$ rust-analyzer analysis-stats .
💀 💀 💀
```

It is especially useful when the `repo` doesn’t use external crates or
the standard library.

If you want to go as far as to modify the source code to debug the
problem, be sure to take a look at the [dev
docs](https://github.com/rust-lang/rust-analyzer/tree/master/docs/dev)!

# Configuration

**Source:**
[config.rs](https://github.com/rust-lang/rust-analyzer/blob/master/crates/rust-analyzer/src/config.rs)

The [Installation](#_installation) section contains details on
configuration for some of the editors. In general `rust-analyzer` is
configured via LSP messages, which means that it’s up to the editor to
decide on the exact format and location of configuration files.

Some clients, such as [VS Code](#vs-code) or [COC plugin in
Vim](#coc-rust-analyzer) provide `rust-analyzer` specific configuration
UIs. Others may require you to know a bit more about the interaction
with `rust-analyzer`.

For the later category, it might help to know that the initial
configuration is specified as a value of the `initializationOptions`
field of the [`InitializeParams` message, in the LSP
protocol](https://microsoft.github.io/language-server-protocol/specifications/specification-current/#initialize).
The spec says that the field type is `any?`, but `rust-analyzer` is
looking for a JSON object that is constructed using settings from the
list below. Name of the setting, ignoring the `rust-analyzer.` prefix,
is used as a path, and value of the setting becomes the JSON property
value.

For example, a very common configuration is to enable proc-macro
support, can be achieved by sending this JSON:

``` json
{
  "cargo": {
    "buildScripts": {
      "enable": true,
    },
  },
  "procMacro": {
    "enable": true,
  }
}
```

Please consult your editor’s documentation to learn more about how to
configure [LSP
servers](https://microsoft.github.io/language-server-protocol/).

To verify which configuration is actually used by `rust-analyzer`, set
`RA_LOG` environment variable to `rust_analyzer=info` and look for
config-related messages. Logs should show both the JSON that
`rust-analyzer` sees as well as the updated config.

This is the list of config options `rust-analyzer` supports:

{{#include generated_config.md}}

# Non-Cargo Based Projects

rust-analyzer does not require Cargo. However, if you use some other
build system, you’ll have to describe the structure of your project for
rust-analyzer in the `rust-project.json` format:

``` TypeScript
interface JsonProject {
    /// Path to the directory with *source code* of
    /// sysroot crates.
    ///
    /// It should point to the directory where std,
    /// core, and friends can be found:
    ///
    /// https://github.com/rust-lang/rust/tree/master/library.
    ///
    /// If provided, rust-analyzer automatically adds
    /// dependencies on sysroot crates. Conversely,
    /// if you omit this path, you can specify sysroot
    /// dependencies yourself and, for example, have
    /// several different "sysroots" in one graph of
    /// crates.
    sysroot_src?: string;
    /// The set of crates comprising the current
    /// project. Must include all transitive
    /// dependencies as well as sysroot crate (libstd,
    /// libcore and such).
    crates: Crate[];
}

interface Crate {
    /// Optional crate name used for display purposes,
    /// without affecting semantics. See the `deps`
    /// key for semantically-significant crate names.
    display_name?: string;
    /// Path to the root module of the crate.
    root_module: string;
    /// Edition of the crate.
    edition: "2015" | "2018" | "2021";
    /// Dependencies
    deps: Dep[];
    /// Should this crate be treated as a member of
    /// current "workspace".
    ///
    /// By default, inferred from the `root_module`
    /// (members are the crates which reside inside
    /// the directory opened in the editor).
    ///
    /// Set this to `false` for things like standard
    /// library and 3rd party crates to enable
    /// performance optimizations (rust-analyzer
    /// assumes that non-member crates don't change).
    is_workspace_member?: boolean;
    /// Optionally specify the (super)set of `.rs`
    /// files comprising this crate.
    ///
    /// By default, rust-analyzer assumes that only
    /// files under `root_module.parent` can belong
    /// to a crate. `include_dirs` are included
    /// recursively, unless a subdirectory is in
    /// `exclude_dirs`.
    ///
    /// Different crates can share the same `source`.
    ///
    /// If two crates share an `.rs` file in common,
    /// they *must* have the same `source`.
    /// rust-analyzer assumes that files from one
    /// source can't refer to files in another source.
    source?: {
        include_dirs: string[],
        exclude_dirs: string[],
    },
    /// The set of cfgs activated for a given crate, like
    /// `["unix", "feature=\"foo\"", "feature=\"bar\""]`.
    cfg: string[];
    /// Target triple for this Crate.
    ///
    /// Used when running `rustc --print cfg`
    /// to get target-specific cfgs.
    target?: string;
    /// Environment variables, used for
    /// the `env!` macro
    env: { [key: string]: string; },

    /// Whether the crate is a proc-macro crate.
    is_proc_macro: boolean;
    /// For proc-macro crates, path to compiled
    /// proc-macro (.so file).
    proc_macro_dylib_path?: string;
}

interface Dep {
    /// Index of a crate in the `crates` array.
    crate: number,
    /// Name as should appear in the (implicit)
    /// `extern crate name` declaration.
    name: string,
}
```

This format is provisional and subject to change. Specifically, the
`roots` setup will be different eventually.

There are three ways to feed `rust-project.json` to rust-analyzer:

- Place `rust-project.json` file at the root of the project, and
  rust-analyzer will discover it.

- Specify
  `"rust-analyzer.linkedProjects": [ "path/to/rust-project.json" ]` in
  the settings (and make sure that your LSP client sends settings as a
  part of initialize request).

- Specify
  `"rust-analyzer.linkedProjects": [ { "roots": […​], "crates": […​] }]`
  inline.

Relative paths are interpreted relative to `rust-project.json` file
location or (for inline JSON) relative to `rootUri`.

See <https://github.com/rust-analyzer/rust-project.json-example> for a
small example.

You can set the `RA_LOG` environment variable to `rust_analyzer=info` to
inspect how rust-analyzer handles config and project loading.

Note that calls to `cargo check` are disabled when using
`rust-project.json` by default, so compilation errors and warnings will
no longer be sent to your LSP client. To enable these compilation errors
you will need to specify explicitly what command rust-analyzer should
run to perform the checks using the `checkOnSave.overrideCommand`
configuration. As an example, the following configuration explicitly
sets `cargo check` as the `checkOnSave` command.

``` json
{ "rust-analyzer.checkOnSave.overrideCommand": ["cargo", "check", "--message-format=json"] }
```

The `checkOnSave.overrideCommand` requires the command specified to
output json error messages for rust-analyzer to consume. The
`--message-format=json` flag does this for `cargo check` so whichever
command you use must also output errors in this format. See the
[Configuration](#_configuration) section for more information.

# Security

At the moment, rust-analyzer assumes that all code is trusted. Here is a
**non-exhaustive** list of ways to make rust-analyzer execute arbitrary
code:

- proc macros and build scripts are executed by default

- `.cargo/config` can override `rustc` with an arbitrary executable

- `rust-toolchain.toml` can override `rustc` with an arbitrary
  executable

- VS Code plugin reads configuration from project directory, and that
  can be used to override paths to various executables, like `rustfmt`
  or `rust-analyzer` itself.

- rust-analyzer’s syntax trees library uses a lot of `unsafe` and hasn’t
  been properly audited for memory safety.

# Privacy

The LSP server performs no network access in itself, but runs
`cargo metadata` which will update or download the crate registry and
the source code of the project dependencies. If enabled (the default),
build scripts and procedural macros can do anything.

The Code extension does not access the network.

Any other editor plugins are not under the control of the
`rust-analyzer` developers. For any privacy concerns, you should check
with their respective developers.

For `rust-analyzer` developers, `cargo xtask release` uses the GitHub
API to put together the release notes.

# Features

{{#include generated_features.md}}

# Assists (Code Actions)

Assists, or code actions, are small local refactorings, available in a
particular context. They are usually triggered by a shortcut or by
clicking a light bulb icon in the editor. Cursor position or selection
is signified by `┃` character.

{{#include generated_assists.md:2:}}

# Diagnostics

While most errors and warnings provided by rust-analyzer come from the
`cargo check` integration, there’s a growing number of diagnostics
implemented using rust-analyzer’s own analysis. Some of these
diagnostics don’t respect `#[allow]` or `\#[deny]` attributes yet, but
can be turned off using the `rust-analyzer.diagnostics.enable`,
`rust-analyzer.diagnostics.experimental.enable` or
`rust-analyzer.diagnostics.disabled` settings.

{{#include generated_diagnostics.md:2:}}

# Editor Features

## VS Code

### Color configurations

It is possible to change the foreground/background color and font
family/size of inlay hints. Just add this to your `settings.json`:

``` jsonc
{
  "editor.inlayHints.fontFamily": "Courier New",
  "editor.inlayHints.fontSize": 11,

  "workbench.colorCustomizations": {
    // Name of the theme you are currently using
    "[Default Dark+]": {
      "editorInlayHint.foreground": "#868686f0",
      "editorInlayHint.background": "#3d3d3d48",

      // Overrides for specific kinds of inlay hints
      "editorInlayHint.typeForeground": "#fdb6fdf0",
      "editorInlayHint.parameterForeground": "#fdb6fdf0",
    }
  }
}
```

### Semantic style customizations

You can customize the look of different semantic elements in the source
code. For example, mutable bindings are underlined by default and you
can override this behavior by adding the following section to your
`settings.json`:

``` jsonc
{
  "editor.semanticTokenColorCustomizations": {
    "rules": {
      "*.mutable": {
        "fontStyle": "", // underline is the default
      },
    }
  },
}
```

Most themes doesn’t support styling unsafe operations differently yet.
You can fix this by adding overrides for the rules `operator.unsafe`,
`function.unsafe`, and `method.unsafe`:

``` jsonc
{
   "editor.semanticTokenColorCustomizations": {
         "rules": {
             "operator.unsafe": "#ff6600",
             "function.unsafe": "#ff6600",
             "method.unsafe": "#ff6600"
         }
    },
}
```

In addition to the top-level rules you can specify overrides for
specific themes. For example, if you wanted to use a darker text color
on a specific light theme, you might write:

``` jsonc
{
   "editor.semanticTokenColorCustomizations": {
         "rules": {
             "operator.unsafe": "#ff6600"
         },
         "[Ayu Light]": {
            "rules": {
               "operator.unsafe": "#572300"
            }
         }
    },
}
```

Make sure you include the brackets around the theme name. For example,
use `"[Ayu Light]"` to customize the theme Ayu Light.

### Special `when` clause context for keybindings.

You may use `inRustProject` context to configure keybindings for rust
projects only. For example:

``` json
{
  "key": "ctrl+alt+d",
  "command": "rust-analyzer.openDocs",
  "when": "inRustProject"
}
```

More about `when` clause contexts
[here](https://code.visualstudio.com/docs/getstarted/keybindings#_when-clause-contexts).

### Setting runnable environment variables

You can use "rust-analyzer.runnableEnv" setting to define runnable
environment-specific substitution variables. The simplest way for all
runnables in a bunch:

``` jsonc
"rust-analyzer.runnableEnv": {
    "RUN_SLOW_TESTS": "1"
}
```

Or it is possible to specify vars more granularly:

``` jsonc
"rust-analyzer.runnableEnv": [
    {
        // "mask": null, // null mask means that this rule will be applied for all runnables
        env: {
             "APP_ID": "1",
             "APP_DATA": "asdf"
        }
    },
    {
        "mask": "test_name",
        "env": {
             "APP_ID": "2", // overwrites only APP_ID
        }
    }
]
```

You can use any valid regular expression as a mask. Also note that a
full runnable name is something like **run bin_or_example_name**, **test
some::mod::test_name** or **test-mod some::mod**, so it is possible to
distinguish binaries, single tests, and test modules with this masks:
`"^run"`, `"^test "` (the trailing space matters!), and `"^test-mod"`
respectively.

### Compiler feedback from external commands

Instead of relying on the built-in `cargo check`, you can configure Code
to run a command in the background and use the `$rustc-watch` problem
matcher to generate inline error markers from its output.

To do this you need to create a new [VS Code
Task](https://code.visualstudio.com/docs/editor/tasks) and set
`rust-analyzer.checkOnSave.enable: false` in preferences.

For example, if you want to run
[`cargo watch`](https://crates.io/crates/cargo-watch) instead, you might
add the following to `.vscode/tasks.json`:

``` json
{
    "label": "Watch",
    "group": "build",
    "type": "shell",
    "command": "cargo watch",
    "problemMatcher": "$rustc-watch",
    "isBackground": true
}
```

### Live Share

VS Code Live Share has partial support for rust-analyzer.

Live Share *requires* the official Microsoft build of VS Code, OSS
builds will not work correctly.

The host’s rust-analyzer instance will be shared with all guests joining
the session. The guests do not have to have the rust-analyzer extension
installed for this to work.

If you are joining a Live Share session and *do* have rust-analyzer
installed locally, commands from the command palette will not work
correctly since they will attempt to communicate with the local server.
