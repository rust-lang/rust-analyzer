use hir::HirDisplay;
use ide_db::{assists::AssistId, famous_defs::FamousDefs};
use syntax::{AstNode, ast};

use crate::assist_context::{AssistContext, Assists};

// Assist: convert_cast_to_from
//
// Convert an `as` cast to a `From` method call.
//
// ```
// //- minicore: from
// impl From<i32> for i64 {
//     fn from(value: i32) -> Self {
//         value as i64
//     }
// }
//
// fn main() {
//     let a: i32 = 3;
//     let b = a a$0s i64;
// }
// ```
// ->
// ```
// impl From<i32> for i64 {
//     fn from(value: i32) -> Self {
//         value as i64
//     }
// }
//
// fn main() {
//     let a: i32 = 3;
//     let b = i64::from(a);
// }
// ```
pub(crate) fn convert_cast_to_from(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let cast_expr = ctx.find_node_at_offset::<ast::CastExpr>()?;
    let inner_expr = cast_expr.expr()?;
    let target_ty_syntax = cast_expr.ty()?;

    let db = ctx.db();
    let sema = &ctx.sema;
    let scope = sema.scope(cast_expr.syntax())?;

    let inner_type = sema.type_of_expr(&inner_expr)?.adjusted();

    let target_type = sema.resolve_type(&target_ty_syntax)?;

    if inner_type.contains_unknown() || target_type.contains_unknown() {
        return None;
    }

    let from_trait = FamousDefs(sema, scope.krate()).core_convert_From()?;

    if !target_type.impls_trait(db, from_trait, &[inner_type]) {
        return None;
    }

    let target_type_str = target_type.display_source_code(db, scope.module().into(), true).ok()?;

    acc.add(
        AssistId::generate("convert_cast_to_from"),
        "Convert `as` to `From`",
        cast_expr.syntax().text_range(),
        |builder| {
            let replacement = if target_type_str.chars().all(|c| c.is_alphanumeric() || c == ':') {
                format!("{target_type_str}::from({inner_expr})")
            } else {
                format!("<{target_type_str}>::from({inner_expr})")
            };
            builder.replace(cast_expr.syntax().text_range(), replacement);
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::convert_cast_to_from;

    #[test]
    fn convert_i32_to_i64() {
        // i32 -> i64 is a widening conversion, From<i32> is implemented for i64
        check_assist(
            convert_cast_to_from,
            r#"
//- minicore: from
impl From<i32> for i64 {
    fn from(value: i32) -> Self {
        value as i64
    }
}

fn main() {
    let a: i32 = 3;
    let b = a a$0s i64;
}
"#,
            r#"
impl From<i32> for i64 {
    fn from(value: i32) -> Self {
        value as i64
    }
}

fn main() {
    let a: i32 = 3;
    let b = i64::from(a);
}
"#,
        );
    }

    #[test]
    fn convert_u8_to_u32() {
        // u8 -> u32 is a widening conversion, From<u8> is implemented for u32
        check_assist(
            convert_cast_to_from,
            r#"
//- minicore: from
impl From<u8> for u32 {
    fn from(value: u8) -> Self {
        value as u32
    }
}

fn main() {
    let a: u8 = 3;
    let b = a a$0s u32;
}
"#,
            r#"
impl From<u8> for u32 {
    fn from(value: u8) -> Self {
        value as u32
    }
}

fn main() {
    let a: u8 = 3;
    let b = u32::from(a);
}
"#,
        );
    }

    #[test]
    fn convert_u8_to_i16() {
        // u8 -> i16 is a lossless conversion (u8 fits in i16), From<u8> is implemented for i16
        check_assist(
            convert_cast_to_from,
            r#"
//- minicore: from
impl From<u8> for i16 {
    fn from(value: u8) -> Self {
        value as i16
    }
}

fn main() {
    let a: u8 = 255;
    let b = a a$0s i16;
}
"#,
            r#"
impl From<u8> for i16 {
    fn from(value: u8) -> Self {
        value as i16
    }
}

fn main() {
    let a: u8 = 255;
    let b = i16::from(a);
}
"#,
        );
    }

    #[test]
    fn convert_with_complex_expr() {
        check_assist(
            convert_cast_to_from,
            r#"
//- minicore: from
impl From<i32> for i64 {
    fn from(value: i32) -> Self {
        value as i64
    }
}

fn main() {
    let b = (1i32 + 2) a$0s i64;
}
"#,
            r#"
impl From<i32> for i64 {
    fn from(value: i32) -> Self {
        value as i64
    }
}

fn main() {
    let b = i64::from((1i32 + 2));
}
"#,
        );
    }

    #[test]
    fn convert_char_to_u32() {
        // char -> u32 is valid, From<char> is implemented for u32
        check_assist(
            convert_cast_to_from,
            r#"
//- minicore: from
impl From<char> for u32 {
    fn from(value: char) -> Self {
        value as u32
    }
}

fn main() {
    let c = 'a';
    let n = c a$0s u32;
}
"#,
            r#"
impl From<char> for u32 {
    fn from(value: char) -> Self {
        value as u32
    }
}

fn main() {
    let c = 'a';
    let n = u32::from(c);
}
"#,
        );
    }

    #[test]
    fn convert_bool_to_i32() {
        // bool -> i32 is valid via as, and From<bool> is implemented for i32
        check_assist(
            convert_cast_to_from,
            r#"
//- minicore: from
impl From<bool> for i32 {
    fn from(value: bool) -> Self {
        value as i32
    }
}

fn main() {
    let b = true;
    let n = b a$0s i32;
}
"#,
            r#"
impl From<bool> for i32 {
    fn from(value: bool) -> Self {
        value as i32
    }
}

fn main() {
    let b = true;
    let n = i32::from(b);
}
"#,
        );
    }

    #[test]
    fn not_applicable_for_narrowing_cast() {
        // i64 -> i32 is a narrowing conversion, no From impl exists
        // (this would require TryFrom)
        check_assist_not_applicable(
            convert_cast_to_from,
            r#"
//- minicore: from
fn main() {
    let a: i64 = 3;
    let b = a a$0s i32;
}
"#,
        );
    }

    #[test]
    fn not_applicable_for_pointer_cast() {
        // Pointer casts don't have From implementations
        check_assist_not_applicable(
            convert_cast_to_from,
            r#"
fn main() {
    let p: *const i32 = 0 as *const i32;
    let n = p a$0s usize;
}
"#,
        );
    }

    #[test]
    fn convert_same_type() {
        // Same type cast - From<T> for T is implemented in minicore
        check_assist(
            convert_cast_to_from,
            r#"
//- minicore: from
fn main() {
    let a: i32 = 3;
    let b = a a$0s i32;
}
"#,
            r#"
fn main() {
    let a: i32 = 3;
    let b = i32::from(a);
}
"#,
        );
    }

    #[test]
    fn convert_in_function_call() {
        check_assist(
            convert_cast_to_from,
            r#"
//- minicore: from
impl From<i32> for i64 {
    fn from(value: i32) -> Self {
        value as i64
    }
}

fn takes_i64(x: i64) {}

fn main() {
    let a: i32 = 3;
    takes_i64(a a$0s i64);
}
"#,
            r#"
impl From<i32> for i64 {
    fn from(value: i32) -> Self {
        value as i64
    }
}

fn takes_i64(x: i64) {}

fn main() {
    let a: i32 = 3;
    takes_i64(i64::from(a));
}
"#,
        );
    }

    #[test]
    fn convert_in_binary_expr() {
        check_assist(
            convert_cast_to_from,
            r#"
//- minicore: from
impl From<i32> for i64 {
    fn from(value: i32) -> Self {
        value as i64
    }
}

fn main() {
    let a: i32 = 3;
    let b: i64 = 5;
    let c = (a a$0s i64) + b;
}
"#,
            r#"
impl From<i32> for i64 {
    fn from(value: i32) -> Self {
        value as i64
    }
}

fn main() {
    let a: i32 = 3;
    let b: i64 = 5;
    let c = (i64::from(a)) + b;
}
"#,
        );
    }

    #[test]
    fn convert_literal() {
        check_assist(
            convert_cast_to_from,
            r#"
//- minicore: from
impl From<i32> for i64 {
    fn from(value: i32) -> Self {
        value as i64
    }
}

fn main() {
    let b = 42i32 a$0s i64;
}
"#,
            r#"
impl From<i32> for i64 {
    fn from(value: i32) -> Self {
        value as i64
    }
}

fn main() {
    let b = i64::from(42i32);
}
"#,
        );
    }
}
