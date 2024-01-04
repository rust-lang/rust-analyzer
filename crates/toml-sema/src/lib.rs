#![allow(dead_code)]
use semver;
use serde::Deserializer;
use std::collections::HashMap;
use toml_syntax::dom;

struct Semantics {
    dict: HashMap<dom::Keys, Value>,
}

struct KeyValuePair {
    key: dom::Keys,
    value: Value,
}

struct Optional<T> {
    default: T,
}

enum Value<T = ValueInner> {
    Optional(Optional<T>),
    Mandatory(T),
}

enum ValueInner {
    Bool(bool),
    Integer(Integer),
    String(String),
    Semver(semver::Version),
    SemverReq(semver::VersionReq),
}

enum Integer {
    RangedInteger(Ranged<i32>),
    Integer(i32),
}

struct Ranged<T> {
    lower: T,
    upper: T,
}

enum String<T> {
    
    EitherOf(T),
}

fn str_to_keys(s: &'_ str) -> dom::Keys {
    let subkeys = s.split(".").into_iter().map(|sfix| {
        assert!(!sfix.is_empty());
        dom::KeyOrIndex::Key(dom::node::Key::new(sfix))
    });
    dom::Keys::new(subkeys)
}

impl Semantics {
    fn new(kvs: Vec<(&str, Value)>) -> Semantics {
        Semantics { dict: kvs.into_iter().map(|kv| (str_to_keys(kv.0), kv.1)).collect() }
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use toml_syntax::dom::{node::Key, KeyOrIndex, Keys};

    use crate::{KeyValuePair, Optional, Semantics, Value};

    fn test_1() {
        let a = r#"
[assist]
emitMustUse = true
expressionFillDefault = "todo"

[cargo]
buildScripts.enable = true

[cargo.buildScripts]
invocationLocation = "workspace"                
"#;
        let parsed = toml_syntax::parse_toml(a);
        let dom = parsed.into_dom();

        #[derive(Deserialize, Debug, Clone)]
        #[serde(rename_all = "snake_case")]
        enum ExprFillDefaultDef {
            Todo,
            Default,
        }

        let kv1 = ("assist.emitMustUse", Value::Optional(Optional { default: false }));
        let kv2 = ( "assist.expressionFillDefault" , Value::Optional(Optional { default:  }));

        let sema = Semantics::new(vec![]);
    }
}
