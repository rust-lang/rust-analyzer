use ra_syntax::{ast, ast::LiteralKind, AstNode, SmolStr};
use std::fmt;

use crate::{Assist, AssistCtx, AssistId};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum NumberLiteralType {
    /// A literal without prefix, '42'
    Decimal,
    /// Hexadecimal literal, '0x2A'
    Hexadecimal,
    /// Octal literal, '0o52'
    Octal,
    /// Binary literal, '0b00101010'
    Binary,
}

#[derive(Clone, Debug)]
struct NumberLiteral {
    /// The type of literal (no prefix, hex, octal or binary)
    number_type: NumberLiteralType,
    /// The suffix as a string, for example 'i32'
    suffix: Option<SmolStr>,
    /// The prefix as string, for example '0x'
    prefix: Option<SmolStr>,
    /// Text of the literal
    text: String,
}

impl fmt::Display for NumberLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(prefix) = &self.prefix {
            f.write_str(prefix)?;
        }

        f.write_str(&self.text)?;

        if let Some(suffix) = &self.suffix {
            f.write_str(suffix)?;
        }

        Ok(())
    }
}

fn identify_number_literal(literal: &ast::Literal) -> Option<NumberLiteral> {
    match literal.kind() {
        LiteralKind::IntNumber { suffix } => {
            let token = literal.token();
            let full_text = token.text().as_str();
            let suffix_clone = suffix.clone();
            let suffix_len = suffix.map(|s| s.len()).unwrap_or_default();
            let non_suffix = &full_text[0..full_text.len() - suffix_len];
            let maybe_prefix = if non_suffix.len() < 2 { None } else { Some(&non_suffix[0..2]) };
            let (prefix, number_type) = match maybe_prefix {
                Some("0x") => (maybe_prefix, NumberLiteralType::Hexadecimal),
                Some("0b") => (maybe_prefix, NumberLiteralType::Binary),
                Some("0o") => (maybe_prefix, NumberLiteralType::Octal),
                _ => (None, NumberLiteralType::Decimal),
            };
            let prefix_len = prefix.map(|s| s.len()).unwrap_or_default();
            let text = &non_suffix[prefix_len..];

            let result = NumberLiteral {
                number_type,
                suffix: suffix_clone,
                prefix: prefix.map(SmolStr::new),
                text: text.to_string(),
            };
            Some(result)
        }
        _ => None,
    }
}

fn is_int_number(literal: &ast::Literal) -> bool {
    match literal.kind() {
        LiteralKind::IntNumber { .. } => true,
        _ => false,
    }
}

fn remove_separator_from_string(str: &str) -> String {
    str.replace("_", "")
}

pub(crate) fn remove_digit_separators(ctx: AssistCtx) -> Option<Assist> {
    let literal = ctx.find_covering_node_at_offset::<ast::Literal>()?;
    if !is_int_number(&literal) {
        return None;
    }

    if !literal.syntax().text().contains_char('_') {
        return None;
    }

    ctx.add_assist(AssistId("remove_digit_separators"), "Remove digit separators", |edit| {
        edit.target(literal.syntax().text_range());
        let new_text = remove_separator_from_string(&literal.syntax().text().to_string());
        edit.replace(literal.syntax().text_range(), new_text);
    })
}

fn len_without_separators(text: &str) -> usize {
    let mut len = 0;
    for c in text.chars() {
        if c != '_' {
            len += 1;
        }
    }
    return len;
}

fn separate_number(text: &str, every: usize, digits_len: usize) -> String {
    let mut result = String::with_capacity(digits_len + digits_len / every);
    let offset = every - (digits_len % every);
    let mut i = 0;
    for c in text.chars() {
        if c != '_' {
            if (i != 0) && ((i + offset) % every == 0) {
                result.push('_');
            }
            result.push(c);
            i += 1;
        }
    }

    return result;
}

#[derive(Clone, Debug)]
struct PossibleSeparateNumberAssist {
    id: AssistId,
    label: String,
    every: usize,
}

const SEPARATE_DECIMAL_THOUSANDS_ID: AssistId = AssistId("separate_decimal_thousands");
const SEPARATE_HEXADECIMAL_WORDS_ID: AssistId = AssistId("separate_hexadecimal_words");
const SEPARATE_HEXADECIMAL_BYTES_ID: AssistId = AssistId("separate_hexadecimal_bytes");
const SEPARATE_BINARY_BYTES_ID: AssistId = AssistId("separate_binary_bytes");
const SEPARATE_BINARY_NIBBLES_ID: AssistId = AssistId("separate_binary_nibbles");

fn get_possible_separate_number_assist(
    literal: &NumberLiteral,
) -> Vec<PossibleSeparateNumberAssist> {
    match literal.number_type {
        NumberLiteralType::Decimal => vec![PossibleSeparateNumberAssist {
            id: SEPARATE_DECIMAL_THOUSANDS_ID,
            label: "Separate thousands".to_string(),
            every: 3,
        }],
        NumberLiteralType::Hexadecimal => vec![
            PossibleSeparateNumberAssist {
                id: SEPARATE_HEXADECIMAL_WORDS_ID,
                label: "Separate 16-bits words".to_string(),
                every: 4,
            },
            PossibleSeparateNumberAssist {
                id: SEPARATE_HEXADECIMAL_BYTES_ID,
                label: "Separate bytes".to_string(),
                every: 2,
            },
        ],
        NumberLiteralType::Binary => vec![
            PossibleSeparateNumberAssist {
                id: SEPARATE_BINARY_BYTES_ID,
                label: "Separate bytes".to_string(),
                every: 8,
            },
            PossibleSeparateNumberAssist {
                id: SEPARATE_BINARY_NIBBLES_ID,
                label: "Separate nibbles".to_string(),
                every: 4,
            },
        ],
        _ => Vec::default(),
    }
}

pub(crate) fn separate_number_literal(ctx: AssistCtx) -> Option<Assist> {
    let literal = ctx.find_covering_node_at_offset::<ast::Literal>()?;
    let number_literal = identify_number_literal(&literal)?;
    let possible_assists = get_possible_separate_number_assist(&number_literal);
    if possible_assists.len() == 0 {
        return None;
    }

    let mut assists = ctx.add_assists();
    for possible_assist in possible_assists {
        let digits_len = len_without_separators(number_literal.text.as_str());
        if digits_len <= possible_assist.every {
            continue;
        }

        let result =
            separate_number(number_literal.text.as_str(), possible_assist.every, digits_len);
        if result == number_literal.text.as_str() {
            continue;
        }

        assists.add_assist(possible_assist.id, possible_assist.label, |edit| {
            edit.target(literal.syntax().text_range());
            let new_literal = NumberLiteral { text: result, ..number_literal.clone() };
            let new_text = new_literal.to_string();
            edit.replace(literal.syntax().text_range(), new_text);
        })
    }

    assists.finish()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::helpers::{
        check_assist, check_assist_not_applicable, check_assist_not_applicable_with_id,
        check_assist_target, check_assist_target_with_id, check_assist_with_id,
    };

    #[test]
    fn remove_digit_separators_target() {
        check_assist_target(
            remove_digit_separators,
            r#"fn f() { let x = <|>42_420; }"#,
            r#"42_420"#,
        );
    }

    #[test]
    fn remove_digit_separators_target_range_inside() {
        check_assist_target(
            remove_digit_separators,
            r#"fn f() { let x = 42<|>_<|>420; }"#,
            r#"42_420"#,
        );
    }

    #[test]
    fn remove_digit_separators_not_applicable_no_separator() {
        check_assist_not_applicable(remove_digit_separators, r#"fn f() { let x = <|>42420; }"#);
    }

    #[test]
    fn remove_digit_separators_not_applicable_range_ends_after() {
        check_assist_not_applicable(remove_digit_separators, r#"fn f() { let x = <|>42_420; <|>}"#);
    }

    #[test]
    fn remove_digit_separators_works_decimal() {
        check_assist(
            remove_digit_separators,
            r#"fn f() { let x = <|>42_420; }"#,
            r#"fn f() { let x = <|>42420; }"#,
        )
    }

    #[test]
    fn remove_digit_separators_works_hex() {
        check_assist(
            remove_digit_separators,
            r#"fn f() { let x = <|>0x42_420; }"#,
            r#"fn f() { let x = <|>0x42420; }"#,
        )
    }

    #[test]
    fn remove_digit_separators_works_octal() {
        check_assist(
            remove_digit_separators,
            r#"fn f() { let x = <|>0o42_420; }"#,
            r#"fn f() { let x = <|>0o42420; }"#,
        )
    }

    #[test]
    fn remove_digit_separators_works_binary() {
        check_assist(
            remove_digit_separators,
            r#"fn f() { let x = <|>0b0010_1010; }"#,
            r#"fn f() { let x = <|>0b00101010; }"#,
        )
    }

    #[test]
    fn remove_digit_separators_works_suffix() {
        check_assist(
            remove_digit_separators,
            r#"fn f() { let x = <|>42_420u32; }"#,
            r#"fn f() { let x = <|>42420u32; }"#,
        )
    }

    // ---

    fn separate_number_for_test(text: &str, every: usize) -> String {
        separate_number(text, every, len_without_separators(text))
    }

    #[test]
    fn test_separate_number() {
        assert_eq!(separate_number_for_test("", 2), "");
        assert_eq!(separate_number_for_test("1", 2), "1");
        assert_eq!(separate_number_for_test("12", 2), "12");
        assert_eq!(separate_number_for_test("12345678", 2), "12_34_56_78");
        assert_eq!(separate_number_for_test("123456789", 2), "1_23_45_67_89");
        assert_eq!(separate_number_for_test("1_2_3_4_5_6_7_8_9", 2), "1_23_45_67_89");

        assert_eq!(separate_number_for_test("", 4), "");
        assert_eq!(separate_number_for_test("1", 4), "1");
        assert_eq!(separate_number_for_test("1212", 4), "1212");
        assert_eq!(separate_number_for_test("24204242420", 4), "242_0424_2420");
        assert_eq!(separate_number_for_test("024204242420", 4), "0242_0424_2420");
        assert_eq!(separate_number_for_test("_0_2_4_2_04242_420", 4), "0242_0424_2420");
    }

    // ---

    #[test]
    fn separate_number_literal_decimal_target() {
        check_assist_target(separate_number_literal, r#"fn f() { let x = <|>42420; }"#, r#"42420"#);
    }

    #[test]
    fn separate_number_literal_decimal_already_split_not_applicable() {
        check_assist_not_applicable(separate_number_literal, r#"fn f() { let x = <|>42_420;}"#);
    }

    #[test]
    fn separate_number_literal_decimal_too_small_not_applicable() {
        check_assist_not_applicable(separate_number_literal, r#"fn f() { let x = <|>420;}"#);
    }

    #[test]
    fn separate_number_literal_decimal_too_small_separator_not_applicable() {
        check_assist_not_applicable(separate_number_literal, r#"fn f() { let x = <|>4_2_0;}"#);
    }

    #[test]
    fn separate_number_literal_decimal() {
        check_assist(
            separate_number_literal,
            r#"fn f() { let x = <|>2420420; }"#,
            r#"fn f() { let x = <|>2_420_420; }"#,
        )
    }

    #[test]
    fn separate_number_literal_decimal_badly_split() {
        check_assist(
            separate_number_literal,
            r#"fn f() { let x = <|>4_2_4_2_0420; }"#,
            r#"fn f() { let x = <|>42_420_420; }"#,
        )
    }

    // ---

    #[test]
    fn separate_number_literal_hex_words_target() {
        check_assist_target_with_id(
            separate_number_literal,
            SEPARATE_HEXADECIMAL_WORDS_ID,
            r#"fn f() { let x = <|>0x04242420; }"#,
            r#"0x04242420"#,
        );
    }

    #[test]
    fn separate_number_literal_hex_words_already_split_not_applicable() {
        check_assist_not_applicable_with_id(
            separate_number_literal,
            SEPARATE_HEXADECIMAL_WORDS_ID,
            r#"fn f() { let x = <|>0x0424_2420; <|>}"#,
        );
    }

    #[test]
    fn separate_number_literal_hex_words_too_small_not_applicable() {
        check_assist_not_applicable_with_id(
            separate_number_literal,
            SEPARATE_HEXADECIMAL_WORDS_ID,
            r#"fn f() { let x = <|>0x2420;}"#,
        );
    }

    #[test]
    fn separate_number_literal_hex_words_too_small_separator_not_applicable() {
        check_assist_not_applicable_with_id(
            separate_number_literal,
            SEPARATE_HEXADECIMAL_WORDS_ID,
            r#"fn f() { let x = <|>0x2_4_2_0;}"#,
        );
    }

    #[test]
    fn separate_number_literal_hex_words() {
        check_assist_with_id(
            separate_number_literal,
            SEPARATE_HEXADECIMAL_WORDS_ID,
            r#"fn f() { let x = <|>0x24204242420; }"#,
            r#"fn f() { let x = <|>0x242_0424_2420; }"#,
        )
    }

    #[test]
    fn separate_number_literal_hex_words_badly_split() {
        check_assist_with_id(
            separate_number_literal,
            SEPARATE_HEXADECIMAL_WORDS_ID,
            r#"fn f() { let x = <|>0x2_4204_24_2420; }"#,
            r#"fn f() { let x = <|>0x242_0424_2420; }"#,
        )
    }

    // ---

    #[test]
    fn separate_number_literal_hex_bytes_target() {
        check_assist_target_with_id(
            separate_number_literal,
            SEPARATE_HEXADECIMAL_BYTES_ID,
            r#"fn f() { let x = <|>0x04242420; }"#,
            r#"0x04242420"#,
        );
    }

    #[test]
    fn separate_number_literal_hex_bytes_already_split_not_applicable() {
        check_assist_not_applicable_with_id(
            separate_number_literal,
            SEPARATE_HEXADECIMAL_BYTES_ID,
            r#"fn f() { let x = <|>0x04_24_24_20; <|>}"#,
        );
    }

    #[test]
    fn separate_number_literal_hex_bytes_too_small_not_applicable() {
        check_assist_not_applicable_with_id(
            separate_number_literal,
            SEPARATE_HEXADECIMAL_BYTES_ID,
            r#"fn f() { let x = <|>0x20;}"#,
        );
    }

    #[test]
    fn separate_number_literal_hex_bytes_too_small_separator_not_applicable() {
        check_assist_not_applicable_with_id(
            separate_number_literal,
            SEPARATE_HEXADECIMAL_BYTES_ID,
            r#"fn f() { let x = <|>0x2_0;}"#,
        );
    }

    #[test]
    fn separate_number_literal_hex_bytes() {
        check_assist_with_id(
            separate_number_literal,
            SEPARATE_HEXADECIMAL_BYTES_ID,
            r#"fn f() { let x = <|>0x24204242420; }"#,
            r#"fn f() { let x = <|>0x2_42_04_24_24_20; }"#,
        )
    }

    #[test]
    fn separate_number_literal_hex_bytes_badly_split() {
        check_assist_with_id(
            separate_number_literal,
            SEPARATE_HEXADECIMAL_BYTES_ID,
            r#"fn f() { let x = <|>0x2_4_2_04242420; }"#,
            r#"fn f() { let x = <|>0x2_42_04_24_24_20; }"#,
        )
    }

    // ---

    #[test]
    fn separate_number_literal_octal_not_applicable() {
        check_assist_not_applicable(
            separate_number_literal,
            r#"fn f() { let x = <|>0o01234567; }"#,
        );
    }

    // ---

    #[test]
    fn separate_number_literal_binary_nibbles_target() {
        check_assist_target_with_id(
            separate_number_literal,
            SEPARATE_BINARY_NIBBLES_ID,
            //r#"fn f() { let x = <|>0b00101010; }"#,
            r#"fn f() { let x = <|>0b00101010; }"#,
            r#"0b00101010"#,
        );
    }

    #[test]
    fn separate_number_literal_binary_nibbles_already_split_not_applicable() {
        check_assist_not_applicable_with_id(
            separate_number_literal,
            SEPARATE_BINARY_NIBBLES_ID,
            r#"fn f() { let x = <|>0b0010_1010_0010_1010; <|>}"#,
        );
    }

    #[test]
    fn separate_number_literal_binary_nibbles_too_small_not_applicable() {
        check_assist_not_applicable_with_id(
            separate_number_literal,
            SEPARATE_BINARY_NIBBLES_ID,
            r#"fn f() { let x = <|>0b1010;}"#,
        );
    }

    #[test]
    fn separate_number_literal_binary_nibbles_too_small_separator_not_applicable() {
        check_assist_not_applicable_with_id(
            separate_number_literal,
            SEPARATE_BINARY_NIBBLES_ID,
            r#"fn f() { let x = <|>0b1_01_0;}"#,
        );
    }

    #[test]
    fn separate_number_literal_binary_nibbles() {
        check_assist_with_id(
            separate_number_literal,
            SEPARATE_BINARY_NIBBLES_ID,
            r#"fn f() { let x = <|>0b0010101000101010; }"#,
            r#"fn f() { let x = <|>0b0010_1010_0010_1010; }"#,
        )
    }

    #[test]
    fn separate_number_literal_binary_nibbles_badly_split() {
        check_assist_with_id(
            separate_number_literal,
            SEPARATE_BINARY_NIBBLES_ID,
            r#"fn f() { let x = <|>0b001_0101_000_101_010; }"#,
            r#"fn f() { let x = <|>0b0010_1010_0010_1010; }"#,
        )
    }

    // ---

    #[test]
    fn separate_number_literal_binary_bytes_target() {
        check_assist_target_with_id(
            separate_number_literal,
            SEPARATE_BINARY_BYTES_ID,
            r#"fn f() { let x = <|>0b0010101000101010; }"#,
            r#"0b0010101000101010"#,
        );
    }

    #[test]
    fn separate_number_literal_binary_bytes_already_split_not_applicable() {
        check_assist_not_applicable_with_id(
            separate_number_literal,
            SEPARATE_BINARY_BYTES_ID,
            r#"fn f() { let x = <|>0b00101010_00101010; <|>}"#,
        );
    }

    #[test]
    fn separate_number_literal_binary_bytes_too_small_not_applicable() {
        check_assist_not_applicable_with_id(
            separate_number_literal,
            SEPARATE_BINARY_BYTES_ID,
            r#"fn f() { let x = <|>0b00101010;}"#,
        );
    }

    #[test]
    fn separate_number_literal_binary_bytes_too_small_separator_not_applicable() {
        check_assist_not_applicable_with_id(
            separate_number_literal,
            SEPARATE_BINARY_BYTES_ID,
            r#"fn f() { let x = <|>0b0_0_101_01_0;}"#,
        );
    }

    #[test]
    fn separate_number_literal_binary_bytes() {
        check_assist_with_id(
            separate_number_literal,
            SEPARATE_BINARY_BYTES_ID,
            r#"fn f() { let x = <|>0b0010101000101010; }"#,
            r#"fn f() { let x = <|>0b00101010_00101010; }"#,
        )
    }

    #[test]
    fn separate_number_literal_binary_bytes_badly_split() {
        check_assist_with_id(
            separate_number_literal,
            SEPARATE_BINARY_BYTES_ID,
            r#"fn f() { let x = <|>0b001_0101_000_101_010; }"#,
            r#"fn f() { let x = <|>0b00101010_00101010; }"#,
        )
    }
}
