//! Rust language whitespace helpers.

//! This matches Rust's definition (Pattern_White_Space)

pub fn is_rust_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{000A}' // line feed (\n)
            | '\u{000B}' // vertical tab
            | '\u{000C}' // form feed
            | '\u{000D}' // carriage return (\r)
            | '\u{0085}' // next line (from latin1)
            | '\u{2028}' // line separator
            | '\u{2029}' // paragraph separator
            // `Default_Ignorable_Code_Point` characters
            | '\u{200E}' // left-to-right mark
            | '\u{200F}' // right-to-left mark
            // Horizontal space characters
            | '\u{0009}' // tab (\t)
            | '\u{0020}' // space
    )
}

pub fn is_rust_horizontal_whitespace(c: char) -> bool {
    matches!(
        c,
        // Horizontal space characters
        '\u{0009}'   // \t
            | '\u{0020}' // space
    )
}

pub fn strip_rust_whitespace(s: &str) -> String {
    s.chars().filter(|&c| !is_rust_whitespace(c)).collect()
}

#[cfg(test)]
mod tests {
    use super::{is_rust_whitespace, strip_rust_whitespace};

    #[test]
    fn rust_whitespace_includes_vertical_tab() {
        assert!(is_rust_whitespace('\u{000B}'));
        assert_eq!(strip_rust_whitespace("a\u{000B}b"), "ab");
    }

    #[test]
    fn rust_whitespace_does_not_include_non_pattern_whitespace() {
        assert!(!is_rust_whitespace('\u{00A0}'));
        assert_eq!(strip_rust_whitespace("a\u{00A0}b"), "a\u{00A0}b");
    }
}
