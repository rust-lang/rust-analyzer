//! ProofPlumber APIs
//! 
//! ProofPlumber APIs include functions inside each module.
//! 
//! These APIs provide ways to utilize functionalities
//! for a developer to write proof actions
//! 
//! See [`crate::AssistContext`] for APIs that utilize context information 
//! For the rest, see [`verus_error`] and [`vst_ext`].
//! 
//! Aside from above APIs, there are several traits already implemented for TOST nodes
//! 
//! TOST nodes (aka VST node) implement various traits including `TryFrom` and `Display`
//! `TryFrom` is used to convert CST node into a TOST Node
//! `Display` is used to convert a TOST node into a concrete code text.
//! 
//! 
//! `new` function is implemented for all TOST nodes.
//! `From` is implemented between various TOST nodes to make it easy to convert between those.
//! For example,
//! ```rust
//! for e in split_exprs {
//!     let split_assert = AssertExpr::new(e);
//!     stmts.statements.push(split_assert.into()); // convert AssertExpr into Stmt
//! }
//! ```
//! 
//! 
//! For Verus syntax definition, read `ungram` file at `syntax` crate.
//!  
//! We use 
//! `syntax/src/tests/sourcegen_vst.rs` to auto-generate `syntax/src/ast/generated/vst_nodes.rs`
//! hand-written pars are at `syntax/src/vst.rs`
//! 
//! 
pub mod proof_action_context;
pub mod verus_error;
pub mod run_fmt;
pub mod vst_ext;
pub mod vst_from_text;
pub mod run_verus;
pub mod inline_function_api;
pub mod semantic_info;