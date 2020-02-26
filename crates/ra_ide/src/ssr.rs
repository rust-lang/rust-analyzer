//!  structural search replace

use crate::source_change::SourceFileEdit;
use ra_ide_db::RootDatabase;
use ra_syntax::ast::make::expr_from_text;
use ra_syntax::{AstNode, SyntaxElement, SyntaxKind, SyntaxNode, WalkEvent};
use ra_text_edit::{TextEdit, TextEditBuilder};
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::str::FromStr;

pub use ra_db::{SourceDatabase, SourceDatabaseExt};
use ra_ide_db::symbol_index::SymbolsDatabase;

#[derive(Debug, PartialEq)]
pub struct SsrError(String);

impl std::fmt::Display for SsrError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Parse error: {}", self.0)
    }
}

impl std::error::Error for SsrError {}

pub fn parse_search_replace(
    query: &str,
    db: &RootDatabase,
) -> Result<Vec<SourceFileEdit>, SsrError> {
    let mut edits = vec![];
    let query: SsrQuery = query.parse()?;
    for &root in db.local_roots().iter() {
        let sr = db.source_root(root);
        for file_id in sr.walk() {
            let matches = find(&query.pattern, db.parse(file_id).tree().syntax());
            if !matches.matches.is_empty() {
                edits.push(SourceFileEdit { file_id, edit: replace(&matches, &query.template) });
            }
        }
    }
    Ok(edits)
}

#[derive(Debug)]
struct SsrQuery {
    pattern: SsrPattern,
    template: SsrTemplate,
}

#[derive(Debug)]
struct SsrPattern {
    pattern: SyntaxNode,
    vars: Vec<Var>,
}

/// represents an `$var` in an SSR query
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Var(String);

#[derive(Debug)]
struct SsrTemplate {
    template: SyntaxNode,
    placeholders: FxHashMap<SyntaxNode, Var>,
}

type Binding = HashMap<Var, SyntaxNode>;

#[derive(Debug)]
struct Match {
    place: SyntaxNode,
    binding: Binding,
}

#[derive(Debug)]
struct SsrMatches {
    matches: Vec<Match>,
}

impl FromStr for SsrQuery {
    type Err = SsrError;

    fn from_str(query: &str) -> Result<SsrQuery, SsrError> {
        let mut it = query.split("==>>");
        let pattern = it.next().expect("at least empty string").trim();
        let mut template = it
            .next()
            .ok_or_else(|| SsrError("Cannot find delemiter `==>>`".into()))?
            .trim()
            .to_string();
        if it.next().is_some() {
            return Err(SsrError("More than one delimiter found".into()));
        }
        let mut vars = vec![];
        let mut it = pattern.split('$');
        let mut pattern = it.next().expect("something").to_string();

        for part in it.map(split_by_var) {
            let (var, var_type, remainder) = part?;
            is_expr(var_type)?;
            let new_var = create_name(var, &mut vars)?;
            pattern.push_str(new_var);
            pattern.push_str(remainder);
            template = replace_in_template(template, var, new_var);
        }

        let template = expr_from_text(&template).syntax().clone();
        let mut placeholders = FxHashMap::default();

        traverse(&template, &mut |n| {
            if let Some(v) = vars.iter().find(|v| v.0.as_str() == n.text()) {
                placeholders.insert(n.clone(), v.clone());
                false
            } else {
                true
            }
        });

        let pattern = SsrPattern { pattern: expr_from_text(&pattern).syntax().clone(), vars };
        let template = SsrTemplate { template, placeholders };
        Ok(SsrQuery { pattern, template })
    }
}

fn traverse(node: &SyntaxNode, go: &mut impl FnMut(&SyntaxNode) -> bool) {
    if !go(node) {
        return;
    }
    for ref child in node.children() {
        traverse(child, go);
    }
}

fn split_by_var(s: &str) -> Result<(&str, &str, &str), SsrError> {
    let end_of_name = s.find(':').ok_or_else(|| SsrError("Use $<name>:expr".into()))?;
    let name = &s[0..end_of_name];
    is_name(name)?;
    let type_begin = end_of_name + 1;
    let type_length =
        s[type_begin..].find(|c| !char::is_ascii_alphanumeric(&c)).unwrap_or_else(|| s.len());
    let type_name = &s[type_begin..type_begin + type_length];
    Ok((name, type_name, &s[type_begin + type_length..]))
}

fn is_name(s: &str) -> Result<(), SsrError> {
    if s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        Ok(())
    } else {
        Err(SsrError("Name can contain only alphanumerics and _".into()))
    }
}

fn is_expr(s: &str) -> Result<(), SsrError> {
    if s == "expr" {
        Ok(())
    } else {
        Err(SsrError("Only $<name>:expr is supported".into()))
    }
}

fn replace_in_template(template: String, var: &str, new_var: &str) -> String {
    let name = format!("${}", var);
    template.replace(&name, new_var)
}

fn create_name<'a>(name: &str, vars: &'a mut Vec<Var>) -> Result<&'a str, SsrError> {
    let sanitized_name = format!("__search_pattern_{}", name);
    if vars.iter().any(|a| a.0 == sanitized_name) {
        return Err(SsrError(format!("Name `{}` repeats more than once", name)));
    }
    vars.push(Var(sanitized_name));
    Ok(&vars.last().unwrap().0)
}

fn find(pattern: &SsrPattern, code: &SyntaxNode) -> SsrMatches {
    let kind = pattern.pattern.kind();
    let matches = code
        .descendants()
        .filter(|n| n.kind() == kind)
        .filter_map(|code| check_match(pattern, &code))
        .collect();
    SsrMatches { matches }
}

fn check_match(pattern: &SsrPattern, code: &SyntaxNode) -> Option<Match> {
    let mut match_ = Match { place: code.clone(), binding: HashMap::new() };
    let pattern_it = &mut pattern.pattern.preorder_with_tokens();
    let code_it = &mut code.preorder_with_tokens();

    loop {
        let pattern_element = get_next_non_whitespace(pattern_it);
        if pattern_element.is_none() {
            return Some(match_);
        }
        let pattern_element = pattern_element.unwrap();
        let code_element = get_next_non_whitespace(code_it)?;

        match (&pattern_element, &code_element) {
            (SyntaxElement::Token(pattern_token), SyntaxElement::Token(code_token)) => {
                if pattern_token.text() != code_token.text() {
                    return None;
                }
            }
            (SyntaxElement::Node(pattern_node), SyntaxElement::Node(code_node)) => {
                if pattern.vars.iter().any(|n| n.0.as_str() == pattern_node.text()) {
                    match_.binding.insert(Var(pattern_node.text().to_string()), code_node.clone());
                    skip_all_children(pattern_it);
                    skip_all_children(code_it);
                } else if pattern_node.kind() != code_node.kind() {
                    return None
                }
            }
            _ => return None,
        }
    }
}

fn replace(matches: &SsrMatches, template: &SsrTemplate) -> TextEdit {
    let mut builder = TextEditBuilder::default();
    for match_ in &matches.matches {
        builder.replace(match_.place.text_range(), render_replace(&match_.binding, template));
    }
    builder.finish()
}

fn render_replace(binding: &Binding, template: &SsrTemplate) -> String {
    let mut builder = TextEditBuilder::default();
    for element in template.template.descendants() {
        if let Some(var) = template.placeholders.get(&element) {
            builder.replace(element.text_range(), binding[var].to_string())
        }
    }
    builder.finish().apply(&template.template.text().to_string())
}

fn skip_all_children<I>(it: &mut I)
where
    I: Iterator<Item = WalkEvent<SyntaxElement>>,
{
    let mut count = 0;
    while let Some(e) = it.next() {
        match e {
            WalkEvent::Enter(_) => count += 1,
            WalkEvent::Leave(_) => {
                if count == 0 {
                    return;
                }
                count -= 1;
            }
        }
    }
}

fn get_next_non_whitespace<I>(it: &mut I) -> Option<SyntaxElement>
where
    I: Iterator<Item = WalkEvent<SyntaxElement>>,
{
    while let Some(event) = it.next() {
        if let WalkEvent::Enter(element) = event {
            if element.kind() != SyntaxKind::WHITESPACE {
                return Some(element);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use ra_syntax::SourceFile;

    fn parse_error_text(query: &str) -> String {
        format!("{}", query.parse::<SsrQuery>().unwrap_err())
    }

    #[test]
    fn parser_happy_case() {
        let result: SsrQuery = "foo($a:expr, $b:expr) ==>> bar($b, $a)".parse().unwrap();
        assert_eq!(&result.pattern.pattern.text(), "foo(__search_pattern_a, __search_pattern_b)");
        assert_eq!(result.pattern.vars.len(), 2);
        assert_eq!(result.pattern.vars[0].0, "__search_pattern_a");
        assert_eq!(result.pattern.vars[1].0, "__search_pattern_b");
        assert_eq!(&result.template.template.text(), "bar(__search_pattern_b, __search_pattern_a)");
    }

    #[test]
    fn parser_empty_query() {
        assert_eq!(parse_error_text(""), "Parse error: Cannot find delemiter `==>>`");
    }

    #[test]
    fn parser_no_delimiter() {
        assert_eq!(parse_error_text("foo()"), "Parse error: Cannot find delemiter `==>>`");
    }

    #[test]
    fn parser_two_delimiters() {
        assert_eq!(
            parse_error_text("foo() ==>> a ==>> b "),
            "Parse error: More than one delimiter found"
        );
    }

    #[test]
    fn parser_no_pattern_type() {
        assert_eq!(parse_error_text("foo($a) ==>>"), "Parse error: Use $<name>:expr");
    }

    #[test]
    fn parser_invalid_name() {
        assert_eq!(
            parse_error_text("foo($a+:expr) ==>>"),
            "Parse error: Name can contain only alphanumerics and _"
        );
    }

    #[test]
    fn parser_invalid_type() {
        assert_eq!(
            parse_error_text("foo($a:ident) ==>>"),
            "Parse error: Only $<name>:expr is supported"
        );
    }

    #[test]
    fn parser_repeated_name() {
        assert_eq!(
            parse_error_text("foo($a:expr, $a:expr) ==>>"),
            "Parse error: Name `a` repeats more than once"
        );
    }

    #[test]
    fn parse_match_replace() {
        let query: SsrQuery = "foo ($x:expr) ==>> bar($x)".parse().unwrap();
        let input = "fn main() { foo( 1 + 2 ); foo (1,2); }";

        let code = SourceFile::parse(input).tree();
        dbg!(&code);
        let matches = find(&query.pattern, code.syntax());
        assert_eq!(matches.matches.len(), 1);
        assert_eq!(matches.matches[0].place.text(), "foo( 1 + 2 )");
        assert_eq!(matches.matches[0].binding.len(), 1);
        assert_eq!(
            matches.matches[0].binding[&Var("__search_pattern_x".to_string())].text(),
            "1 + 2"
        );

        let edit = replace(&matches, &query.template);
        assert_eq!(edit.apply(input), "fn main() { bar(1 + 2); foo (1,2); }");
    }
}
