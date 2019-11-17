//! Internal utility functions.

use ra_syntax::{ast, ast::AstToken, SourceFile};

/// Unquotes text.
pub(crate) fn unquote_str(s: &str) -> Option<String> {
    let parsed = SourceFile::parse(s);
    let quoted = parsed.syntax_node().descendants_with_tokens().find_map(|it| it.into_token())?;
    let maybe_str = ast::String::cast(quoted.clone()).and_then(|it| it.value());

    maybe_str.or_else(|| ast::RawString::cast(quoted).and_then(|it| it.value()))
}
