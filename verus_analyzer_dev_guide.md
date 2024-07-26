
### Using cargo watch
Developing verus-analyzer can be time-consuming when it needs to interact with VS Code. In that case, we can use `cargo watch` to automatically build and run the analyzer. 

```bash
#! /usr/bin/bash
cargo xtask dist
gunzip dist/rust-analyzer-x86_64-unknown-linux-gnu.gz
chmod +x dist/rust-analyzer-x86_64-unknown-linux-gnu
```

In a separate tmux pane, run this with `cargo watch -- ./build_gunzip_chmod.sh`

When it finishes, do `Developer: Reload Window` in VS Code to reload verus-analyzer.


#### How to update verus-analyzer when Verus syntax changes

#### Summary:
1. Add a testcase at `syntax/src/lib.rs`.
2. Update `syntax/rust.ungram` file and modify `syntax/src/ast/tests/ast_src.rs` if necessary.
3. Run `cargo test --package syntax --lib -- tests::sourcegen_ast --nocapture` to auto-generate `syntax/ast/generated/*` files.
4. Update `parser` crate to parse new syntax item.
5. Test that proof actions still work by running:
```
cargo test --package ide-assists --lib -- handlers::proof_action
```
This currently requires setting `TMPDIR` and `VERUS_BINARY_PATH`

#### Details:

##### Checking Verus syntax changes
- Before making changes to verus-analyzer, refer `verus/dependencies/syn` crate to check how Verus handles new syntax item. Although there are many differences in `syn` and rust-analzer, but it is helpful to keep them as similar as possible. 
For example, inside `verus/dependencies/syn/src/items.rs`, refer `impl parse for Signature` to see how Verus function's signature is parsed. 

- For additional syntax information, refer `verus/source/builtin_macros/src/syntax.rs` 

- `verus/source/rust_verify/examples/syntax.rs` contains syntax examples that can be handy for testcases. 


##### Modifying `syntax` and `parser` crates
Inside the `crates` directory, we need to modify several crates, but most changes will be made on `parser` and `syntax` crates.

1. Update `syntax/rust.ungram` with the new syntax. Also, update `syntax/tests/ast_src.rs` for newly introduced tokens if there is any. 

2. Use `syntax/tests/sourcegen_ast.rs` to auto-generate `syntax/ast/generated/*` files. They use `rust.ungram` and `ast_src.rs` as input. (e.g. `cargo test --package syntax --lib -- tests::sourcegen_ast --nocapture `)

3. Add testcases. Add snippets of new Verus code at `syntax/src/lib.rs`, to make sure the new syntax is parsed correctly. `workflows/verus.yml` will run these tests in the CI.

4. To modify the parser, start from `syntax/src/grammar/verus.rs`. Verus specific lang items(e.g. `requires` `ensures`) should be parsed here. For modified items (e.g. `WhileExpr`), the parser is modified in-place. See `item.rs` and `expression.rs` for examples of these. The implicit rule is that for each “ungrammar” object, there is a function that parses that object. 

    For example:
    - For `AssertExpr`, we have `grammar::verus::assert` function to parse it. 
    - For `struct`, there is  `grammar::items::adt::struckt` function to parse struct.
    - For major syntax items, refer to `grammar/item.rs` file.

5. Test that proof actions still work by running:
```
cargo test --package ide-assists --lib -- handlers::proof_action
```
This currently requires setting the `TMPDIR` and `VERUS_BINARY_PATH` environment variables

##### Modifying the rest

Modify `hir-def` and `hit-ty` crates if necessary. The changes will be alerted
by the compiler("missing enum case"), and they can be largely straight forward.
These changes are needed for the IDE purposes(e.g. type inference, code
scanning, etc).  The best approach is to find an existing piece of syntax
similar to the one you added and mimic it.


