//! Generates preinterned names.

use std::{env, path::Path};

macro_rules! x {
    ( $($name:tt = $string:literal,)+ ) => {
        static ATOMS: &[&str] = &[
            $($string),+
        ];
    };
}

include!("known_names.rs");

fn main() {
    string_cache_codegen::AtomType::new("name::RaAtom", "ra_atom!")
        .atoms(ATOMS)
        .write_to_file(&Path::new(&env::var("OUT_DIR").unwrap()).join("ra_atom.rs"))
        .unwrap();
}
