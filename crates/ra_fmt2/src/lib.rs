//! More forgiving rust fmt ra-fmt2
// TODO remove when done
#![allow(dead_code)]
#![allow(unused_imports)]

mod dsl;
mod edit_tree;
mod engine;
mod pattern;
#[macro_use]
mod rules;
mod scratch;
mod trav_util;
mod whitespace;

pub use engine::{format_str};
