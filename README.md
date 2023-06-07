## Using VS Code for Verus

**(WARNING: this is experimental and under active development)**

The steps below walk you through compiling a Verus-specific version of rust-analyzer and using it in VS Code. It provides Verus syntax support and several IDE functionalities.


### Quick Start

#### 1. Compile custom rust-analyzer

1. Clone the repository: `git clone https://github.com/verus-lang/rust-analyzer.git`
2. `cd rust-analyzer`
3. Compile the rust-analyzer binary: `cargo xtask dist`
4. Unzip the generated file (e.g. `gunzip ./dist/rust-analyzer-x86_64-apple-darwin.gz`)
5. Make it executable (e.g. `chmod +x ./dist/rust-analyzer-x86_64-apple-darwin`)



#### 2. VS Code
Before starting, please install the original rust-analyzer extension in VS Code's extensions tab.

##### 2.1. Adding a separate [VS Code Workspace](https://code.visualstudio.com/docs/editor/workspaces)
Suppose you have a new project with `cargo new`. After you open this project in VS Code, use `File > Save Workspace As...` to generate `{project_name}.code-workspace` file. The file will look similar to this.

```
{
	"folders": [
		{
			"path": "."
		}
	],
	"settings": {}
}
```


##### 2.2. Adding settings variables
We will modify the "settings" section of the `.code-workspace` file. To be specific, we will add two entries in the "settings" section of the file. These are `rust-analyzer.server.path` and `rust-analyzer.check.command`.

- `rust-analyzer.server.path` should be set to the path of the custom rust-analyzer binary produced in step 1 above (e.g., the full path to `./dist/rust-analyzer-x86_64-apple-darwin`)
- `rust-analyzer.check.command` to "" (empty string) to disable cargo check. This is to prevent normal `cargo check` from running when you save the file, which will produce incorrect errors.

For example, the "settings" in the `.code-workspace` file could look the following:
```json
"settings": {
        "rust-analyzer.server.path": "ABSOLUTE-PATH-TO-THE-CUSTOM-BINARY",
        "rust-analyzer.check.command": "",  // to disable cargo check
}
```

When you modify and save this file, VS Code will ask you if you want to reload the rust-analyzer server. It will replace the rust-analyzer binary with this custom one.

By opening this workspace, the rust-analyzer plugin will use the custom binary. If you open your project without that workspace setting(e.g. open this project by "open folder"), it will use the original rust-analyzer binary.


##### 2.3. [VS Code in Remote Machine](https://code.visualstudio.com/docs/remote/ssh)(Optional)
For VS Code in Remote Machine, we need to set up the entire above process in the remote machine. That includes Verus being installed in the same remote machine.




---
### Functionalities and Details

#### 1.Syntax
We extended rust-analyzer's grammar for Verus-specific syntax. This custom rust-analyzer highlights reserved Verus keywords (e.g. `spec`, `proof`, `requires`, `ensures`). If a user types `prooof` instead of `proof`, a syntax error will be generated for it.


#### 2.IDE functionalities
Rust-analyzer scans the project root and all files that are reachable from the root. If the file you are working on is not reachable from the project root, most of the IDE functionalities like "goto definition" will not work. For example, if you open the `verus` repository and open up one of the examples, these functionalities might not work.

You can find more documents for IDE functionalities on the following links.
- [Go to Definition](https://rust-analyzer.github.io/manual.html#go-to-definition)
- [Go to Type Declaration](https://rust-analyzer.github.io/manual.html#go-to-type-definition)
- [Find all References](https://rust-analyzer.github.io/manual.html#find-all-references)
- [Hover](https://rust-analyzer.github.io/manual.html#hover)


---
### Limitations
- This is currently experimental and subject to change.
- It is intended to be used only for Verus code. (For Rust code, you can use the original binary by opening the project without the `.code-workspace` file, which is just using "open folder" in VS Code)
- Some features of rust-analyzer might be broken or missing.
- An issue was reported while compiling this custom rust-analyzer on Apple Silicon Mac. As a temporary measure, running `rustup target add x86_64-apple-darwin` might help bypass the problem.
- Information of `builtin` is currently not known to this custom rust-analyzer. For example, the builtin types like `int` and `nat` could be shown as `unknown`.
- `pervasive`/`vstd` is not scanned
- requires/ensures/decreaes/invariant/assert-by-block/assert-forall-block are not fully scanned for IDE purposes(e.g. might not be able to "goto definition" of the function used in requires/ensures)
