#### How to update verus-analyzer when Verus syntax changes

1. Before making changes to verus-analyzer, check `verus/dependencies/syn` crate for changes. For example, inside `verus/dependencies/syn/src/items.rs`, `impl parse for Signature` implements how Verus function signature is parsed.

2. Update `syntax/rust.ungram` with the new syntax. Also, update `syntax/tests/ast_src.rs` for newly introduced tokens if there is any. 

3. Use `syntax/tests/sourcegen_ast.rs` to auto-generate `syntax/ast/generated/*` files. It uses `ast_src.rs` as input. (e.g. run `cargo test --package syntax --lib -- tests::sourcegen_ast --nocapture `)

4. Add testcases. Add snippets of new Verus code at `syntax/src/lib.rs`, to make sure the new syntax is parsed correctly. `workflows/verus.yml` will run the tests in the CI.

5. Might need to change 1) `syntax/src/ast/prec.rs`, 2)
