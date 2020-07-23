//! ra_cfg defines conditional compiling options, `cfg` attibute parser and evaluator

mod cfg_expr;

use ra_syntax::SmolStr;
use rustc_hash::FxHashSet;

pub use cfg_expr::{parse_cfg, CfgExpr};

/// Configuration options used for conditional compilition on items with `cfg` attributes.
/// We have two kind of options in different namespaces: atomic options like `unix`, and
/// key-value options like `target_arch="x86"`.
///
/// Note that for key-value options, one key can have multiple values (but not none).
/// `feature` is an example. We have both `feature="foo"` and `feature="bar"` if features
/// `foo` and `bar` are both enabled. And here, we store key-value options as a set of tuple
/// of key and value in `key_values`.
///
/// See: https://doc.rust-lang.org/reference/conditional-compilation.html#set-configuration-options
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CfgOptions {
    atoms: FxHashSet<SmolStr>,
    key_values: FxHashSet<(SmolStr, SmolStr)>,
}

impl CfgOptions {
    pub fn check(&self, cfg: &CfgExpr) -> Option<bool> {
        cfg.fold(&|key, value| match value {
            None => self.atoms.contains(key),
            Some(value) => self.key_values.contains(&(key.clone(), value.clone())),
        })
    }

    pub fn is_cfg_enabled(&self, attr: &tt::Subtree) -> Option<bool> {
        self.check(&parse_cfg(attr))
    }

    pub fn insert(&mut self, raw_string: &str) {
        match raw_string.find('=') {
            None => {
                self.atoms.insert(raw_string.into());
            }
            Some(pos) => {
                let key = &raw_string[..pos];
                let value = raw_string[pos + 1..].trim_matches('"');
                self.key_values.insert((key.into(), value.into()));
            }
        }
    }

    pub fn remove(&mut self, name: &str) {
        self.atoms.remove(name);
    }

    pub fn insert_key_value(&mut self, key: SmolStr, value: SmolStr) {
        self.key_values.insert((key, value));
    }

    pub fn append(&mut self, other: &CfgOptions) {
        for atom in &other.atoms {
            self.atoms.insert(atom.clone());
        }

        for (key, value) in &other.key_values {
            self.key_values.insert((key.clone(), value.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfg_options_atom_from_raw() {
        let mut cfg_options = CfgOptions::default();

        cfg_options.insert("debug_assertions");
        cfg_options.insert("unix");

        assert_eq!(Some(true), cfg_options.check(&CfgExpr::Atom("unix".into())));
        assert_eq!(Some(true), cfg_options.check(&CfgExpr::Atom("debug_assertions".into())));

        assert_eq!(Some(false), cfg_options.check(&CfgExpr::Atom("missing_atom".into())));
        assert_eq!(
            Some(false),
            cfg_options
                .check(&CfgExpr::KeyValue { key: "target_family".into(), value: "unix".into() })
        );
    }

    #[test]
    fn test_cfg_options_key_value_from_raw() {
        let mut cfg_options = CfgOptions::default();

        cfg_options.insert(r#"target_arch="x86_64""#);
        cfg_options.insert("target_endian=little");

        assert_eq!(
            Some(true),
            cfg_options
                .check(&CfgExpr::KeyValue { key: "target_arch".into(), value: "x86_64".into() })
        );
        assert_eq!(
            Some(true),
            cfg_options
                .check(&CfgExpr::KeyValue { key: "target_endian".into(), value: "little".into() })
        );

        assert_eq!(
            Some(false),
            cfg_options
                .check(&CfgExpr::KeyValue { key: "target_family".into(), value: "unix".into() })
        );
        assert_eq!(Some(false), cfg_options.check(&CfgExpr::Atom("missing_atom".into())));
    }
}
