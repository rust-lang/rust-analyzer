# Kakoune

[Kakoune](https://kakoune.org/) supports LSP with the help of [kak-lsp](https://github.com/kak-lsp/kak-lsp).
Follow the [instructions](https://github.com/kak-lsp/kak-lsp#installation) to install `kak-lsp`.

To configure `kak-lsp`, refer to the [configuration section](https://github.com/kak-lsp/kak-lsp#configuring-kak-lsp) which is basically about copying the [configuration file](https://github.com/kak-lsp/kak-lsp/blob/master/kak-lsp.toml) in the right place (latest versions should use `rust-analyzer` by default).

Finally, you need to configure Kakoune to talk to `kak-lsp` (see [Usage section](https://github.com/kak-lsp/kak-lsp#usage)).

A basic configuration will only get you LSP but you can also activate inlay diagnostics and auto-formatting on save.
The following might help you get all of this.

```txt
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
