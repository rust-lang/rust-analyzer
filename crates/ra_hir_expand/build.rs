use std::{env, path::Path};

macro_rules! make {
    ($ident:ident) => {
        stringify!($ident)
    };
    ($ident:ident @ $real:tt) => {
        stringify!($real)
    };
}

macro_rules! x {
    ( $($name:tt $(@ $real:tt)?,)+ ) => {
        static ATOMS: &[&str] = &[
            $(make!($name $(@ $real)?)),+
        ];
    };
}

include!("known_names.rs");

fn main() {
    string_cache_codegen::AtomType::new("RaAtom", "ra_atom!")
        .atoms(ATOMS)
        .write_to_file(&Path::new(&env::var("OUT_DIR").unwrap()).join("ra_atom.rs"))
        .unwrap();
}
