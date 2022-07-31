# Vim/NeoVim

Prerequisites: You have installed the [rust-analyzer binary](server_binary.md). binary.
Not needed if the extension can install/update it on its own, coc-rust-analyzer is one example.

There are several LSP client implementations for vim or neovim:

## coc-rust-analyzer

1. Install coc.nvim by following the instructions at
   [coc.nvim](https://github.com/neoclide/coc.nvim)
   (Node.js required)
1. Run `:CocInstall coc-rust-analyzer` to install
   [coc-rust-analyzer](https://github.com/fannheyward/coc-rust-analyzer),
   this extension implements _most_ of the features supported in the VSCode extension:
   * automatically install and upgrade stable/nightly releases
   * same configurations as VSCode extension, `rust-analyzer.server.path`, `rust-analyzer.cargo.features` etc.
   * same commands too, `rust-analyzer.analyzerStatus`, `rust-analyzer.ssr` etc.
   * inlay hints for variables and method chaining, _Neovim Only_

Note: for code actions, use `coc-codeaction-cursor` and `coc-codeaction-selected`; `coc-codeaction` and `coc-codeaction-line` are unlikely to be useful.

## LanguageClient-neovim

1. Install LanguageClient-neovim by following the instructions
   [here](https://github.com/autozimu/LanguageClient-neovim)
   * The GitHub project wiki has extra tips on configuration

2. Configure by adding this to your vim/neovim config file (replacing the existing Rust-specific line if it exists):

```vim
let g:LanguageClient_serverCommands = {
\ 'rust': ['rust-analyzer'],
\ }
```

## YouCompleteMe

Install YouCompleteMe by following the instructions
  [here](https://github.com/ycm-core/YouCompleteMe#installation).

rust-analyzer is the default in ycm, it should work out of the box.

## ALE

To use the LSP server in [ale](https://github.com/dense-analysis/ale):

```vim
let g:ale_linters = {'rust': ['analyzer']}
```

## nvim-lsp

NeoVim 0.5 has built-in language server support.
For a quick start configuration of rust-analyzer, use [neovim/nvim-lspconfig](https://github.com/neovim/nvim-lspconfig#rust_analyzer).
Once `neovim/nvim-lspconfig` is installed, use `+lua require'lspconfig'.rust_analyzer.setup({})+` in your `init.vim`.

You can also pass LSP settings to the server:

```lua
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

See <https://sharksforarms.dev/posts/neovim-rust/> for more tips on getting started.

Check out <https://github.com/simrat39/rust-tools.nvim> for a batteries included rust-analyzer setup for neovim.

## vim-lsp

vim-lsp is installed by following [the plugin instructions](https://github.com/prabirshrestha/vim-lsp).
It can be as simple as adding this line to your `.vimrc`:

```vim
Plug 'prabirshrestha/vim-lsp'
```

Next you need to register the `rust-analyzer` binary.
If it is available in `$PATH`, you may want to add this to your `.vimrc`:

```vim
if executable('rust-analyzer')
  au User lsp_setup call lsp#register_server({
        \   'name': 'Rust Language Server',
        \   'cmd': {server_info->['rust-analyzer']},
        \   'whitelist': ['rust'],
        \ })
endif
```

There is no dedicated UI for the server configuration, so you would need to send any options as a value of the `initialization_options` field, as described in the [Configuration](../configuration.md) section.
Here is an example of how to enable the proc-macro support:

```vim
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
