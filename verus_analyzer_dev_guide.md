
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
1. Add a testcase to `crates/syntax/src/lib.rs`.
2. Update the `crates/syntax/rust.ungram` file and modify `xtask/src/codegen/grammar/ast_src.rs` if necessary.
3. Run `cargo xtask codegen grammar` to  auto-generate `crates/syntax/ast/generated/*` and `crates/parser/src/syntax_kind/generated.rs` files.
4. Update `parser` crate to parse new syntax item.
5. Run the new and existing syntax tests via `cargo test --package syntax --lib`
6. Test that proof actions still work by running:
```
cargo test --package ide-assists --lib -- handlers::proof_action
```
This currently requires setting `TMPDIR` and `VERUS_BINARY_PATH`


#### Details:

##### Checking Verus syntax changes
- Before making changes to verus-analyzer, refer to Verus' `verus/dependencies/syn` crate to check how Verus handles the new syntax item. Although there are many differences between `syn` and rust-analyzer, it is helpful to keep them as similar as possible. 
For example, inside `verus/dependencies/syn/src/items.rs`, refer to `impl parse for Signature` to see how Verus parses a function signature. 

- For additional syntax information, refer to Verus's `verus/source/builtin_macros/src/syntax.rs`.

- `verus/source/rust_verify/examples/syntax.rs` contains syntax examples that can be handy for testcases. 

- `verusfmt` can also be a useful source of grammar documentation; see in particular the `src/verus.pest` file.  It can also provide useful test cases -- see `tests/verus-consistency.rs`


##### Modifying `syntax` and `parser` crates
Inside the `crates` directory, we need to modify several crates, but most changes will be made on the `parser` and `syntax` crates.

1. Add a testcase to `crates/syntax/src/lib.rs`.
2. Update `syntax/rust.ungram` with the new syntax. Also, update `xtask/src/codegen/grammar/ast_src.rs` for newly introduced tokens if there are any. 
  - In particular, you will need to update the `keywords` list for new keywords to be available

3. Run `cargo xtask codegen grammar` to  auto-generate `crates/syntax/ast/generated/*` and `crates/parser/src/syntax_kind/generated.rs` files.
  - This relies on these files `xtask/src/codegen/grammar/{ast_src.rs,sourcegen_vst.rs}` 

4. Add testcases. Add snippets of new Verus code at `syntax/src/lib.rs`, to make sure the new syntax is parsed correctly. `.github/workflows/verus.yml` will run these tests in the CI.

5. To modify the parser, start from `parser/src/grammar/verus.rs`. Verus specific lang items(e.g. `requires` `ensures`) should be parsed here. For modified items (e.g. `WhileExpr`), the parser is modified in-place. See `item.rs` and `expression.rs` for examples of these. The implicit rule is that for each “ungrammar” object, there is a function that parses that object. 

    For example:
    - For `AssertExpr`, we have `grammar::verus::assert` function to parse it. 
    - For `struct`, there is  `grammar::items::adt::struckt` function to parse struct.
    - For major syntax items, refer to `grammar/item.rs` file.

6. Test that proof actions still work by running:
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

