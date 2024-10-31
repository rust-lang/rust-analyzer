# <a href="https://verus-lang.github.io/verus/verus/logo.html"><img height="30px" src="https://verus-lang.github.io/verus/verus/assets/verus-color.svg" alt="Verus" /></a> Verus-Analyzer
Verus-analyzer is a version of [rust-analyzer](https://github.com/rust-lang/rust-analyzer) that has
been modified to provide IDE support for writing [Verus](https://github.com/verus-lang/verus) code 
and proofs, including syntax support and various IDE features.

## WARNING!
This software is **experimental** and subject to change; some features are likely broken.
At present, it works best on small, self-contained Verus projects.  Anything more complex
will likely fail.  You may file issues, but we do not currently have dedicated engineering
support for `verus-analyzer`, so **your issue may not be addressed**.  Pull requests with
fixes are always welcome, although it is unlikely they will be reviewed immediately.

## Quick Start

The intended use of `verus-analyzer` is as a drop-in replacement for `rust-analyzer`.


### 1. Compile binary

**1. Obtain the verus-analyzer source**
```bash
$ git clone https://github.com/verus-lang/verus-analyzer.git
$ cd verus-analyzer
```

**2. Compiler the verus-analyzer binary**
```bash
$ cargo xtask dist
```

**3. Unzip and make the binary executable**
This step is dependent on your operating system.

#### MacOS:
```bash
$ gunzip ./dist/verus-analyzer-aarch64-apple-darwin.gz
$ chmod +x ./dist/verus-analyzer-aarch64-apple-darwin
```

#### Linux:
```bash
$ gunzip dist/verus-analyzer-x86_64-unknown-linux-gnu.gz
$ chmod +x dist/verus-analyzer-x86_64-unknown-linux-gnu
```

#### Windows:

Locate the generated zip file `dist\verus-analyzer-x86_64-pc-windows-msvc.zip` and extract it (e.g.,
through right-click + extract all). Then you should see the file `dist\verus-analyzer-x86_64-pc-windows-msvc\rust-analyzer.exe`
that you are going to use as the verus-analyzer binary.

### 2. IDE Setup

To use `verus-analyzer` in your IDE (e.g., VS Code, EMACS or VIM) you need to configure your IDE to use the compiled `verus-analyzer` binary instead of `rust-analyzer` for `*.rs` files.

#### EMACS Setup

TODO

#### VIM Setup

TODO

#### VS Code Setup

You will need to install the `rust-analyzer` extension
```
code --install-extension rust-lang.rust-analyzer
```

Then you can configure the `rust-analyzer` extensions to use the `verus-analyzer` binary that you compiled. You can do so by either

1. Use a new  [VS Code Workspace](https://code.visualstudio.com/docs/editor/workspaces) and specify the settings, or
2. Adding the settings of your existing workspace through the `.vscode/settings.json` files.

You will need to add the following settings:
```json
{
  "rust-analyzer.server.path": "FULL-PATH-TO-THE-VERUS-ANALYZER-BINARY",
  "rust-analyzer.checkOnSave": false
}
```
Note: the path must be the absolute path to the generated binary. We disable `cargo-check` by setting `rust-analyzer.checkOnSave` to false.

When saving the file, you may get prompted to reload the `rust-analyzer` server.

When opening a Verus project with the created workspace settings or open a folder with the `.vscode/settings.json`, it will use the custom binary. Otherwise, it will keep using the original `rust-analyzer` binary.

**[Remote - SSH]((https://code.visualstudio.com/docs/remote/ssh))** If you are using VS code on a remote machine make sure to setup `verus-analyzer` on the remote machine and configure the settings as mentioned above with the paths on the remote machine.


### 3. Requirements for IDE functionalities

There is a requirement for the IDE functionalities to work. The requirement is that `rust-analyzer` expects a standard Rust project layout and the metadata(`Cargo.toml`) file.

Rust-analyzer scans the project root(`lib.rs` or `main.rs`) and all files that are reachable from the root. If the file you are working on is not reachable from the project root, most of the IDE functionalities like "goto definition" will not work. For example, when you have a file named `foo.rs` next to `main.rs`, and did not import `foo.rs` in `main.rs`(i.e., did not add `mod foo` on top of `main.rs`), the IDE functionalities will not work for `foo.rs`.

We also need `Cargo.toml` files as in standard Rust projects. For a small project, you could start with `cargo new`, and it automatically generated the `Cargo.toml` file. For a larger project, you could use [workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html) to manage multiple crates.


### 4. Running Verus in VS Code (optional)
This is an experimental feature that allows you to run Verus inside VS Code by saving a file.  To run Verus inside VS Code, please remove `"rust-analyzer.checkOnSave": false` from the `.code-workspace` file, and add `"rust-analyzer.checkOnSave.overrideCommand"`.  The value of `"rust-analyzer.checkOnSave.overrideCommand"` should be a list that contains the Verus binary.  Please use the absolute path to the Verus binary that is built by `vargo`.

For example, add the following entries to your workspace settings or the `.vscode/settings.json` file.
```json
"rust-analyzer.server.path": "ABSOLUTE-PATH-TO-THE-VERUS-ANALYZER-BINARY",
"rust-analyzer.checkOnSave.overrideCommand": [
    "ABSOLUTE-PATH-TO-VERUS-BINARY", //  e.g., /home/verus-username/verus/source/target-verus/(debug|release)/verus
]
}
```

To provide extra arguments, add `extra_args` at `[package.metadata.verus.ide]` inside your `Cargo.toml` file. For example, if you want to run Verus with `--crate-type=dylib --expand-errors`, you could add the following at the bottom of your `Cargo.toml` file.
```toml
[package.metadata.verus.ide]
extra_args = "--crate-type=dylib --expand-errors"
```

Please set only one of `rust-analyzer.checkOnSave.overrideCommand` and `rust-analyzer.checkOnSave` in the `.code-workspace` file, depending as to whether you want to run Verus inside VS Code or not.

---
## Functionalities and Details

### 1.Syntax
We extended rust-analyzer's grammar for Verus-specific syntax. This custom rust-analyzer highlights reserved Verus keywords (e.g., `spec`, `proof`, `requires`, `ensures`). If a user types `prof` instead of `proof`, a syntax error will be generated for it.


### 2.IDE functionalities
You can find more documents for IDE functionalities on the following links.
- [Go to Definition](https://rust-analyzer.github.io/manual.html#go-to-definition)
- [Go to Type Declaration](https://rust-analyzer.github.io/manual.html#go-to-type-definition)
- [Find all References](https://rust-analyzer.github.io/manual.html#find-all-references)
- [Hover](https://rust-analyzer.github.io/manual.html#hover)

#### 2.1 TODOs for IDE functionalities
- Code scanning is incomplete for Verus-specific items. To be specific, requires/ensures/decreases/invariant/assert-by-block/assert-forall-block are not fully scanned for IDE purposes (e.g., might not be able to "goto definition" of the function used in requires/ensures, "find all references" might omit occurrences inside requires/ensures).

- Although Verus' custom operators are parsed, thye are not registered for IDE purposes. For example, type inference around such operators might not work. (e.g., `A ==> B` is parsed as `implies(A, B)`, but the IDE might not be able to infer that `A` and `B` are Boolean).

- `builtin` and `vstd` are not scanned. For example, the builtin types like `int` and `nat` could be shown as `unknown`. Auto completion for `vstd` might not work.


---
### Limitations
- This is experimental software and subject to change.
- It is intended to be used only for Verus code. (For Rust code, you can use the original binary by opening the project without the `.code-workspace` file, which is just using "open folder" in VS Code)
- Multiple features of rust-analyzer might be broken or missing.
- Syntax might not be updated to the latest version of Verus.
<!-- - An issue was reported while compiling this custom rust-analyzer on Apple Silicon Mac. As a temporary measure, running `rustup target add x86_64-apple-darwin` might help bypass the problem. -->


#### Misc
- `rust-analyzer: Clear flycheck diagnostics` command can be used to clear the error messages in VS Code
- `Developer: Reload Window` can be used to reload VS Code and the verus-analyzer server instead of closing and reopening VS Code
- Setting `"rust-analyzer.diagnostics.disabled": ["syntax-error"]` in the workspace setting can disable the syntax error messages in VS Code. You could also add `unresolved-module` to the above list to disable the error message for unresolved modules.
- There is no proper support for `buildin`/`vstd`. However, in your project's `Cargo.toml` file, you can add `vstd` in `dependencices` or `dev-dependencies`, which might make verus-analyzer scan `vstd` and `builtin`. For example,
```
[dependencies]
vstd = { path = "../verus/source/vstd"}  # assuming verus and the project are in the same directory
```



---

## Proof Actions (Optional)

Proof actions are an **experimental** feature to assist developers when debugging proof failures.

### Compilation
Compile Verus analyzer by following the steps below.  These steps are similar to the basic version with the exception
of the extra flag used in step 3.

1. Clone the repository: `git clone https://github.com/verus-lang/verus-analyzer.git`
2. `cd verus-analyzer`
3. Compile the rust-analyzer binary: `cargo xtask dist --proof-action`
4. Unzip the generated file (e.g., `gunzip ./dist/verus-analyzer-aarch64-apple-darwin.gz`)
5. Make it executable (e.g., `chmod +x ./dist/verus-analyzer-aarch64-apple-darwin`)


### Prerequisites
* Follow the directions for [4. Running Verus in VS Code (optional)](#4-running-verus-in-vs-code-optional)
* [verusfmt](https://github.com/verus-lang/verusfmt)
You can install `verusfmt` using:
```
cargo install verusfmt --locked
```
You can then use `which verusfmt` to get the absolute path to it.



### Configuration

The "settings" entry in the `.code-workspace` file needs some additional configuration to provide environment variables for the verus-analyzer binary.  In particular, we need to define: "VERUS_BINARY_PATH", "TMPDIR", and "VERUS_FMT_BINARY_PATH".

```json
"rust-analyzer.server.extraEnv": {
        "VERUS_BINARY_PATH" : "ABSOLUTE-PATH-TO-VERUS-BINARY", //  e.g., /home/verus-username/verus/source/target-verus/(debug|release)/verus
        "TMPDIR": "ABSOLUTE-PATH-TO-TMP-DIR", // e.g., "/home/verus-username/.tmpdir"
        "VERUS_FMT_BINARY_PATH" : "ABSOLUTE-PATH-TO-VERUS-FMT", // e.g., "/home/verus-username/.cargo/bin/verusfmt"
}
```

Hence, the final configuration for `settings` might look like the following.

```json
"settings": {
        "rust-analyzer.server.path": "ABSOLUTE-PATH-TO-THE-VERUS-ANALYZER-BINARY",
        "rust-analyzer.server.extraEnv": {
                "VERUS_BINARY_PATH" : "ABSOLUTE-PATH-TO-VERUS-BINARY", //  e.g., /home/verus-username/verus/source/target-verus/(debug|release)/verus
                "TMPDIR": "ABSOLUTE-PATH-TO-TMP-DIR", // e.g., "/home/verus-username/.tmpdir"
                "VERUS_FMT_BINARY_PATH" : "ABSOLUTE-PATH-TO-VERUS-FMT", // e.g., "/home/verus-username/.cargo/bin/verusfmt"
        },
        "rust-analyzer.checkOnSave.overrideCommand": [
            "ABSOLUTE-PATH-TO-VERUS-BINARY", //  e.g., /home/verus-username/verus/source/target-verus/(debug|release)/verus
        ],
}
```

### Proof Action Demo
[Source code](https://github.com/chanheec/proof-action-example)

![](demo.gif)
