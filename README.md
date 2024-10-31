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

**1a. Obtain the verus-analyzer source**
```bash
$ git clone https://github.com/verus-lang/verus-analyzer.git
$ cd verus-analyzer
```

**1b. Compiler the verus-analyzer binary**
```bash
$ cargo xtask dist
```

**1c. Unzip and make the binary executable**
This step is dependent on your operating system.

#### 1c1. MacOS:
```bash
$ gunzip ./dist/verus-analyzer-aarch64-apple-darwin.gz
$ chmod +x ./dist/verus-analyzer-aarch64-apple-darwin
```

#### 1c2. Linux:
```bash
$ gunzip dist/verus-analyzer-x86_64-unknown-linux-gnu.gz
$ chmod +x dist/verus-analyzer-x86_64-unknown-linux-gnu
```

#### 1c3. Windows:

Locate the generated zip file `dist\verus-analyzer-x86_64-pc-windows-msvc.zip` and extract it (e.g.,
through right-click + extract all). Then you should see the file `dist\verus-analyzer-x86_64-pc-windows-msvc\rust-analyzer.exe`
that you are going to use as the verus-analyzer binary.

### 2. Project setup

`rust-analyzer` expects a standard Rust project layout and the metadata(`Cargo.toml`) file. For this quick start, run `cargo new demo`.

### 3. IDE Setup

To use `verus-analyzer` in your IDE (e.g., VS Code, EMACS or VIM) you need to configure your IDE to use the compiled `verus-analyzer` binary instead of `rust-analyzer` for `*.rs` files.

#### 3a. EMACS Setup

TODO

#### 3b. VIM Setup

TODO

#### 3c. VS Code Setup

Install the `rust-analyzer` extension into VSCode:
```
code --install-extension rust-lang.rust-analyzer
```

Configure the `rust-analyzer` extension to use the `verus-analyzer` binary that
you compiled. In this example, we'll configure it for the single `demo` workspace
we created in the previous step.
Create a vscode workspace configuration file in that directory `demo/demo.code-workspace` containing:
```
{
	"folders": [
		{
			"path": "."
		}
	],
	"settings": {
        "rust-analyzer.server.path": "<absolute-path-to-verus-analyzer-build>/dist/verus-analyzer-x86_64-unknown-linux-gnu",
          "rust-analyzer.checkOnSave.overrideCommand": [ "<absolute-path-to-verus-executable>"]
	}
}
```

The `server.path' configuration enables Verus-specifc syntax coloring.
The `checkOnSave` parameter runs Verus on every File->Save operation,
providing Verus errors in the source code with squiggles and hovertext.

Alternatively, to enable only syntax coloring and not Verus verification on save,
replace the `checkOnSave.overrideCommand` line with
```
  "rust-analyzer.checkOnSave": false
```

Open VSCode. Use "File->Open Workspace from File..." and select
the "demo.code-workspace" file you created above. Paste a Verus program
into `main.rs`:
```rust
use vstd::prelude::*;

verus!{
    pub open spec fn fib(x: int) -> int
    decreases x
    {
        if x <= 1 { 1 } else {
          fib(x-2) + fib(x-1)
        }
    }
}

fn main() {
    // println!("Hello, world!");
}
```

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
