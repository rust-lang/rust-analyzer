# rust-analyzer Language Server Binary

Other editors generally require the `rust-analyzer` binary to be in `$PATH`.
You can download pre-built binaries from the [releases](https://github.com/rust-lang/rust-analyzer/releases) page.
You will need to uncompress and rename the binary for your platform, e.g. from `rust-analyzer-aarch64-apple-darwin.gz` on Mac OS to `rust-analyzer`, make it executable, then move it into a directory in your `$PATH`.

On Linux to install the `rust-analyzer` binary into `~/.local/bin`, these commands should work:

```shell
mkdir -p ~/.local/bin
curl -L https://github.com/rust-lang/rust-analyzer/releases/latest/download/rust-analyzer-x86_64-unknown-linux-gnu.gz | gunzip -c - > ~/.local/bin/rust-analyzer
chmod +x ~/.local/bin/rust-analyzer
```

Make sure that `~/.local/bin` is listed in the `$PATH` variable and use the appropriate URL if you're not on a `x86-64` system.

You don't have to use `~/.local/bin`, any other path like `~/.cargo/bin` or `/usr/local/bin` will work just as well.

Alternatively, you can install it from source using the command below.
You'll need the latest stable version of the Rust toolchain.

```shell
git clone https://github.com/rust-lang/rust-analyzer.git && cd rust-analyzer
cargo xtask install --server
```

If your editor can't find the binary even though the binary is on your `$PATH`, the likely explanation is that it doesn't see the same `$PATH` as the shell, see [this issue](https://github.com/rust-lang/rust-analyzer/issues/1811).
On Unix, running the editor from a shell or changing the `.desktop` file to set the environment should help.

## rustup

`rust-analyzer` is available in `rustup`, but only in the nightly toolchain:

```shell
rustup +nightly component add rust-analyzer-preview
```

However, in contrast to `component add clippy` or `component add rustfmt`, this does not actually place a `rust-analyzer` binary in `~/.cargo/bin`, see [this issue](https://github.com/rust-lang/rustup/issues/2411).

## Arch Linux

The `rust-analyzer` binary can be installed from the repos or AUR (Arch User Repository):

* [rust-analyzer](https://www.archlinux.org/packages/community/x86_64/rust-analyzer/) (built from latest tagged source)
* [rust-analyzer-git](https://aur.archlinux.org/packages/rust-analyzer-git) (latest Git version)

Install it with pacman, for example:

```shell
pacman -S rust-analyzer
```

## Gentoo Linux

`rust-analyzer` is available in the GURU repository:

* [dev-util/rust-analyzer](https://gitweb.gentoo.org/repo/proj/guru.git/tree/dev-util/rust-analyzer?id=9895cea62602cfe599bd48e0fb02127411ca6e81) builds from source
* [dev-util/rust-analyzer-bin](https://gitweb.gentoo.org/repo/proj/guru.git/tree/dev-util/rust-analyzer-bin?id=9895cea62602cfe599bd48e0fb02127411ca6e81) installs an official binary release

If not already, GURU must be enabled (e.g. using `app-eselect/eselect-repository`) and sync'd before running `emerge`:

```shell
eselect repository enable guru && emaint sync -r guru
emerge rust-analyzer-bin
```

## macOS

The `rust-analyzer` binary can be installed via [Homebrew](https://brew.sh/).

```shell
brew install rust-analyzer
```
