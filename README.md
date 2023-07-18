# Using VS Code for Verus

**WARNING: this software is experimental and subject to change; some features might be broken**

The steps below walk you through compiling a Verus-specific version of rust-analyzer and using it in VS Code. It provides Verus syntax support and several IDE functionalities.



## Quick Start

### 1. Compile binary

1. Clone the repository: `git clone https://github.com/verus-lang/verus-analyzer.git`
2. `cd verus-analyzer`
3. Compile the rust-analyzer binary: `cargo xtask dist`
4. Unzip the generated file (e.g., `gunzip ./dist/verus-analyzer-x86_64-apple-darwin.gz`)
5. Make it executable (e.g., `chmod +x ./dist/verus-analyzer-x86_64-apple-darwin`)



### 2. VS Code
Before starting, please install the original rust-analyzer extension in VS Code's extensions tab.

#### 2.1. Adding a separate [VS Code Workspace](https://code.visualstudio.com/docs/editor/workspaces)
Suppose you have a new project with `cargo new`. After you open this project in VS Code, use `File > Save Workspace As...` to generate `{project_name}.code-workspace` file. The file will look similar to this.

```json
{
	"folders": [
		{
			"path": "."
		}
	],
	"settings": {}
}
```


#### 2.2. Adding settings variables
We will modify the "settings" section of the `.code-workspace` file. To be specific, we will add two entries in the "settings" section of the file. These are `rust-analyzer.server.path` and `rust-analyzer.checkOnSave`.

- `rust-analyzer.server.path` should be set to the path of the verus-analyzer binary produced in step 1 above (e.g., the full path to `./dist/rust-analyzer-x86_64-apple-darwin`)
- `rust-analyzer.checkOnSave` to disable `cargo check`.

For example, the "settings" in the `.code-workspace` file could look the following:
```json
"settings": {
        "rust-analyzer.server.path": "ABSOLUTE-PATH-TO-THE-VERUS-ANALYZER-BINARY",
        "rust-analyzer.checkOnSave": false,
}
```

When you modify and save this file, VS Code will ask you if you want to reload the rust-analyzer server. It will replace the rust-analyzer binary with this custom one.

By opening this workspace, the rust-analyzer plugin will use the custom binary. If you open your project without that workspace setting(e.g., open this project by "open folder"), it will use the original rust-analyzer binary.


#### 2.3. [VS Code in Remote Machine](https://code.visualstudio.com/docs/remote/ssh)(Optional)
For VS Code in Remote Machine, we need to set up the entire above process in the remote machine. That includes Verus being installed in the same remote machine.


### 3. Requirements for IDE functionalities
There is a requirement for the IDE functionalities to work. The requirement is that Rust-analyzer expects a standard Rust project layout and the metadata(`Cargo.toml`) file.

Rust-analyzer scans the project root(`lib.rs` or `main.rs`) and all files that are reachable from the root. If the file you are working on is not reachable from the project root, most of the IDE functionalities like "goto definition" will not work. For example, when you have a file named `foo.rs` next to `main.rs`, and did not import `foo.rs` in `main.rs`(i.e., did not add `mod foo` on top of `main.rs`), the IDE functionalities will not work for `foo.rs`.

We also need `Cargo.toml` files as in standard Rust projects. For a small project, you could start with `cargo new`, and it automatically generated the `Cargo.toml` file. For a larger project, you could use [workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html) to manage multiple crates.


### 4. Running Verus in VS Code (optional)
This is an experimemntal feature that allows you to run Verus inside VS Code by saving a file.  To run Verus inside VS Code, please remove `"rust-analyzer.checkOnSave": false` from the `.code-workspace` file, and add `"rust-analyzer.checkOnSave.overrideCommand"`.  The value of `"rust-analyzer.checkOnSave.overrideCommand"` should be a list that contains the Verus binary.  Please use the absolute path to the Verus binary that is built by `vargo`.

For example, the "settings" in the `.code-workspace` file could now look the following:
```json
"settings": {
        "rust-analyzer.server.path": "ABSOLUTE-PATH-TO-THE-VERUS-ANALYZER-BINARY",
        "rust-analyzer.checkOnSave.overrideCommand": [
            "ABSOLUTE-PATH-TO-VERUS-BINARY", //  e.g., /home/verus-username/verus/source/target-verus/(debug|release)/verus
        ],
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
We extended rust-analyzer's grammar for Verus-specific syntax. This custom rust-analyzer highlights reserved Verus keywords (e.g., `spec`, `proof`, `requires`, `ensures`). If a user types `prooof` instead of `proof`, a syntax error will be generated for it.


### 2.IDE functionalities
You can find more documents for IDE functionalities on the following links.
- [Go to Definition](https://rust-analyzer.github.io/manual.html#go-to-definition)
- [Go to Type Declaration](https://rust-analyzer.github.io/manual.html#go-to-type-definition)
- [Find all References](https://rust-analyzer.github.io/manual.html#find-all-references)
- [Hover](https://rust-analyzer.github.io/manual.html#hover)

#### 2.1 TODOs for IDE functionalities
- Code scanning is incomplete for Verus-specific items. To be specific, requires/ensures/decreases/invariant/assert-by-block/assert-forall-block are not fully scanned for IDE purposes.(e.g., might not be able to "goto definition" of the function used in requires/ensures, "find all references" might omit occurences inside requires/ensures)

- Although Verus' custom operators are parsed, those are not registered for IDE purposes. For example, type inference around those operators might not work. (e.g., `A ==> B` is parsed as `implies(A, B)`, but the IDE might not be able to infer that `A` and `B` are boolean)

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



