//! This module contains functions for editing syntax trees. As the trees are
//! immutable, all function here return a fresh copy of the tree, instead of
//! doing an in-place modification.
use parser::{SyntaxKind::DOC_COMMENT, T};
use std::{fmt, iter::once, ops};

use crate::{
    SyntaxElement,
    SyntaxKind::{ATTR, NEWLINE, WHITESPACE},
    SyntaxNode, SyntaxToken,
    algo::strip_blank_edges,
    ast::{self, AstNode, HasName, make},
    syntax_editor::{Position, Removable, SyntaxEditor, SyntaxMappingBuilder},
};

use super::syntax_factory::SyntaxFactory;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndentLevel(pub u8);

impl From<u8> for IndentLevel {
    fn from(level: u8) -> IndentLevel {
        IndentLevel(level)
    }
}

impl fmt::Display for IndentLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let spaces = "                                        ";
        let buf;
        let len = self.0 as usize * 4;
        let indent = if len <= spaces.len() {
            &spaces[..len]
        } else {
            buf = " ".repeat(len);
            &buf
        };
        fmt::Display::fmt(indent, f)
    }
}

impl ops::Add<u8> for IndentLevel {
    type Output = IndentLevel;
    fn add(self, rhs: u8) -> IndentLevel {
        IndentLevel(self.0 + rhs)
    }
}

impl ops::AddAssign<u8> for IndentLevel {
    fn add_assign(&mut self, rhs: u8) {
        self.0 += rhs;
    }
}

impl IndentLevel {
    pub fn zero() -> IndentLevel {
        IndentLevel(0)
    }
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
    pub fn from_element(element: &SyntaxElement) -> IndentLevel {
        match element {
            rowan::NodeOrToken::Node(it) => IndentLevel::from_node(it),
            rowan::NodeOrToken::Token(it) => IndentLevel::from_token(it),
        }
    }

    pub fn from_node(node: &SyntaxNode) -> IndentLevel {
        match node.first_non_trivia_token() {
            Some(it) => Self::from_token(&it),
            None => IndentLevel(0),
        }
    }

    pub fn from_token(token: &SyntaxToken) -> IndentLevel {
        let mut start = token.clone();
        while let Some(prev) = start.prev_token().filter(|it| it.kind() != NEWLINE) {
            start = prev;
        }
        let indent = if start.kind() == WHITESPACE { start.text() } else { "" };
        IndentLevel((indent.chars().count() / 4) as u8)
    }

    pub(super) fn clone_increase_indent(self, node: &SyntaxNode) -> SyntaxNode {
        self.clone_adjust_indent(node, true)
    }

    pub(super) fn clone_decrease_indent(self, node: &SyntaxNode) -> SyntaxNode {
        self.clone_adjust_indent(node, false)
    }

    fn clone_adjust_indent(self, node: &SyntaxNode, increase: bool) -> SyntaxNode {
        let (editor, node) = SyntaxEditor::new(node.clone());
        let indent = self.to_string();
        let mut line_start = !increase;
        for token in node.descendants_with_tokens().filter_map(|it| it.into_token()) {
            let text: String = token.leading_trivia().map(|it| it.text().to_owned()).collect();
            let last = text.matches('\n').count();
            let adjusted = text
                .split('\n')
                .enumerate()
                .map(|(index, line)| {
                    if (index == 0 && !line_start) || (index != last && line.is_empty()) {
                        line.to_owned()
                    } else if increase {
                        format!("{indent}{line}")
                    } else {
                        line.strip_prefix(&indent).unwrap_or(line).to_owned()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            if adjusted != text {
                editor.splice_leading_trivia(&token, .., make::tokens::trivia(&adjusted));
            }
            line_start = token.trailing_trivia().next_back().is_some_and(|it| it.kind() == NEWLINE);
        }
        editor.finish().new_root().clone()
    }
}

pub trait AstNodeEdit: AstNode + Clone + Sized {
    fn indent_level(&self) -> IndentLevel {
        IndentLevel::from_node(self.syntax())
    }
    #[must_use]
    fn indent(&self, level: IndentLevel) -> Self {
        Self::cast(level.clone_increase_indent(self.syntax())).unwrap()
    }
    #[must_use]
    fn indent_with_mapping(&self, level: IndentLevel, make: &SyntaxFactory) -> Self {
        let new_node = self.indent(level);
        if let Some(mut mapping) = make.mappings() {
            let mut builder = SyntaxMappingBuilder::new(new_node.syntax().clone());
            for (old, new) in self.syntax().children().zip(new_node.syntax().children()) {
                builder.map_node(old, new);
            }
            builder.finish(&mut mapping);
        }
        new_node
    }
    #[must_use]
    fn detached(&self) -> Self {
        Self::cast(strip_blank_edges(self.syntax(), true)).unwrap()
    }
    #[must_use]
    fn dedent(&self, level: IndentLevel) -> Self {
        Self::cast(level.clone_decrease_indent(self.syntax())).unwrap()
    }
    #[must_use]
    fn reset_indent(&self) -> Self {
        let level = IndentLevel::from_node(self.syntax());
        self.dedent(level)
    }
}

impl<N: AstNode + Clone> AstNodeEdit for N {}

pub trait AttrsOwnerEdit: ast::HasAttrs {
    fn remove_attrs_and_docs(&self, editor: &SyntaxEditor) {
        for child in self.syntax().children_with_tokens() {
            if matches!(child.kind(), ATTR | DOC_COMMENT) {
                editor.delete(child);
            }
        }
    }
}

impl<T: ast::HasAttrs> AttrsOwnerEdit for T {}

impl ast::IdentPat {
    pub fn set_pat(&self, pat: Option<ast::Pat>, editor: &SyntaxEditor) -> ast::IdentPat {
        let make = editor.make();
        match pat {
            None => {
                if let Some(at_token) = self.at_token() {
                    // Remove `@ Pat`
                    let start = at_token.clone().into();
                    let end = self
                        .pat()
                        .map(|it| it.syntax().clone().into())
                        .unwrap_or_else(|| at_token.into());
                    editor.delete_all(start..=end);
                }
            }
            Some(pat) => {
                if let Some(old_pat) = self.pat() {
                    // Replace existing pattern
                    editor.replace(old_pat.syntax(), pat.syntax())
                } else if let Some(at_token) = self.at_token() {
                    // Have an `@` token but not a pattern yet
                    editor.insert(Position::after(at_token), pat.syntax());
                } else {
                    // Don't have an `@`, should have a name
                    let name = self.name().unwrap();
                    let elements = vec![make.token(T![@]).into(), pat.syntax().clone().into()];

                    if self.syntax().parent().is_none() {
                        let (local, local_self) = SyntaxEditor::with_ast_node(self);
                        let local_name = local_self.name().unwrap();
                        local.insert_all(Position::after(local_name.syntax()), elements);
                        let edit = local.finish();
                        return ast::IdentPat::cast(edit.new_root().clone()).unwrap();
                    } else {
                        editor.insert_all(Position::after(name.syntax()), elements);
                    }
                }
            }
        }
        self.clone()
    }
}

impl ast::UseTree {
    pub fn wrap_in_tree_list_with_editor(&self) -> Option<ast::UseTree> {
        if self.use_tree_list().is_some()
            && self.path().is_none()
            && self.star_token().is_none()
            && self.rename().is_none()
        {
            return None;
        }

        let (editor, use_tree) = SyntaxEditor::with_ast_node(self);
        let make = editor.make();
        let first_child = use_tree.syntax().first_child_or_token()?;
        let last_child = use_tree.syntax().last_child_or_token()?;
        let use_tree_list = make.use_tree_list(once(self.clone()));
        editor.replace_all(first_child..=last_child, vec![use_tree_list.syntax().clone().into()]);

        let edit = editor.finish();
        ast::UseTree::cast(edit.new_root().clone())
    }
}

pub fn indent(node: &SyntaxNode, level: IndentLevel) -> SyntaxNode {
    level.clone_increase_indent(node)
}

impl ast::GenericParamList {
    /// Constructs a matching [`ast::GenericArgList`]
    pub fn to_generic_args(&self, make: &SyntaxFactory) -> ast::GenericArgList {
        let args = self.generic_params().filter_map(|param| match param {
            ast::GenericParam::LifetimeParam(it) => {
                Some(ast::GenericArg::LifetimeArg(make.lifetime_arg(it.lifetime()?)))
            }
            ast::GenericParam::TypeParam(it) => {
                Some(ast::GenericArg::TypeArg(make.type_arg(make.ty_name(it.name()?))))
            }
            ast::GenericParam::ConstParam(it) => {
                // Name-only const params get parsed as `TypeArg`s
                Some(ast::GenericArg::TypeArg(make.type_arg(make.ty_name(it.name()?))))
            }
        });

        make::generic_arg_list(args)
    }
}

impl ast::UseTree {
    /// Deletes the usetree node represented by the input. Recursively removes parents, including use nodes that become empty.
    pub fn remove_recursive(self, editor: &SyntaxEditor) {
        let parent = self.syntax().parent();

        if let Some(u) = parent.clone().and_then(ast::Use::cast) {
            u.remove(editor);
        } else if let Some(u) = parent.and_then(ast::UseTreeList::cast) {
            if u.use_trees().nth(1).is_none()
                || u.use_trees().all(|use_tree| {
                    use_tree.syntax() == self.syntax() || editor.deleted(use_tree.syntax())
                })
            {
                u.parent_use_tree().remove_recursive(editor);
                return;
            }
            self.remove(editor);
            u.remove_unnecessary_braces(editor);
        }
    }

    /// Splits off the given prefix, making it the path component of the use tree,
    /// appending the rest of the path to all UseTreeList items.
    ///
    /// # Examples
    ///
    /// `prefix$0::suffix` -> `prefix::{suffix}`
    ///
    /// `prefix$0` -> `prefix::{self}`
    ///
    /// `prefix$0::*` -> `prefix::{*}`
    pub fn split_prefix_with_editor(&self, editor: &SyntaxEditor, prefix: &ast::Path) {
        debug_assert_eq!(self.path(), Some(prefix.top_path()));

        let make = editor.make();
        let path = self.path().unwrap();
        let suffix = if path == *prefix {
            if self.use_tree_list().is_some() {
                return;
            } else if self.star_token().is_some() {
                make.use_tree_glob()
            } else {
                let self_path = make.path_unqualified(make.path_segment_self());
                make.use_tree(self_path, None, self.rename(), false)
            }
        } else {
            let suffix_segments = path.segments().skip(prefix.segments().count());
            let suffix_path = make.path_from_segments(suffix_segments, false);
            make.use_tree(
                suffix_path,
                self.use_tree_list(),
                self.rename(),
                self.star_token().is_some(),
            )
        };
        let use_tree_list = make.use_tree_list(once(suffix));
        let new_use_tree = make.use_tree(prefix.clone(), Some(use_tree_list), None, false);

        editor.replace(self.syntax(), new_use_tree.syntax());
    }
}

impl ast::RecordExprField {
    /// This will either replace the initializer, or in the case that this is a shorthand convert
    /// the initializer into the name ref and insert the expr as the new initializer.
    pub fn replace_expr(&self, editor: &SyntaxEditor, expr: ast::Expr) {
        if self.name_ref().is_some() {
            if let Some(prev) = self.expr() {
                editor.replace(prev.syntax(), expr.syntax());
            }
        } else if let Some(ast::Expr::PathExpr(path_expr)) = self.expr()
            && let Some(path) = path_expr.path()
            && let Some(name_ref) = path.as_single_name_ref()
        {
            // shorthand `{ x }` → expand to `{ x: expr }`
            let new_field = editor
                .make()
                .record_expr_field(editor.make().name_ref(name_ref.text()), Some(expr));
            editor.replace(self.syntax(), new_field.syntax());
        }
    }
}

#[test]
fn test_increase_indent() {
    let arm_list = {
        let arm = make::match_arm(make::wildcard_pat().into(), None, make::ext::expr_unit());
        make::match_arm_list([arm.clone(), arm])
    };
    assert_eq!(
        arm_list.syntax().to_string(),
        "{
    _ => (),
    _ => (),
}"
    );
    let indented = arm_list.indent(IndentLevel(2));
    assert_eq!(
        indented.syntax().to_string(),
        "{
            _ => (),
            _ => (),
        }"
    );
}

#[test]
fn split_prefix_inserts_self() {
    check_split_prefix("use foo;", "foo::{self}");
}

#[test]
fn split_prefix_preserves_rename() {
    check_split_prefix("use foo as bar;", "foo::{self as bar}");
}

#[test]
fn split_prefix_wraps_glob() {
    check_split_prefix("use foo::*;", "foo::{*}");
}

#[cfg(test)]
fn check_split_prefix(before: &str, expected: &str) {
    let source = crate::SourceFile::parse(before, parser::Edition::CURRENT).tree();
    let use_tree = source.syntax().descendants().find_map(ast::UseTree::cast).unwrap();
    let (editor, use_tree) = SyntaxEditor::with_ast_node(&use_tree);
    let prefix = use_tree.path().unwrap();
    use_tree.split_prefix_with_editor(&editor, &prefix);
    let edit = editor.finish();
    assert_eq!(edit.new_root().to_string(), expected);
}
