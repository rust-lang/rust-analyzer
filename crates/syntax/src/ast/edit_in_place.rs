//! Structural editing for ast.

use std::iter::{empty, once, successors};

use parser::{SyntaxKind, T};

use crate::{
    AstNode, AstToken, Direction, SyntaxElement,
    SyntaxKind::{ATTR, COMMENT, WHITESPACE},
    SyntaxNode, SyntaxToken,
    algo::{self, neighbor},
    ast::{self, edit::IndentLevel, make},
    syntax_editor::{self, SyntaxEditor},
    ted,
};

use super::HasName;

pub trait AttrsOwnerEdit: ast::HasAttrs {
    fn remove_attrs_and_docs(&self) {
        remove_attrs_and_docs(self.syntax());

        fn remove_attrs_and_docs(node: &SyntaxNode) {
            let mut remove_next_ws = false;
            for child in node.children_with_tokens() {
                match child.kind() {
                    ATTR | COMMENT => {
                        remove_next_ws = true;
                        child.detach();
                        continue;
                    }
                    WHITESPACE if remove_next_ws => {
                        child.detach();
                    }
                    _ => (),
                }
                remove_next_ws = false;
            }
        }
    }
}

impl<T: ast::HasAttrs> AttrsOwnerEdit for T {}

impl ast::GenericParamList {
    /// Removes the existing generic param
    pub fn remove_generic_param(&self, generic_param: ast::GenericParam) {
        if let Some(previous) = generic_param.syntax().prev_sibling() {
            if let Some(next_token) = previous.next_sibling_or_token() {
                ted::remove_all(next_token..=generic_param.syntax().clone().into());
            }
        } else if let Some(next) = generic_param.syntax().next_sibling() {
            if let Some(next_token) = next.prev_sibling_or_token() {
                ted::remove_all(generic_param.syntax().clone().into()..=next_token);
            }
        } else {
            ted::remove(generic_param.syntax());
        }
    }

    /// Constructs a matching [`ast::GenericArgList`]
    pub fn to_generic_args(&self) -> ast::GenericArgList {
        let args = self.generic_params().filter_map(|param| match param {
            ast::GenericParam::LifetimeParam(it) => {
                Some(ast::GenericArg::LifetimeArg(make::lifetime_arg(it.lifetime()?)))
            }
            ast::GenericParam::TypeParam(it) => {
                Some(ast::GenericArg::TypeArg(make::type_arg(make::ext::ty_name(it.name()?))))
            }
            ast::GenericParam::ConstParam(it) => {
                // Name-only const params get parsed as `TypeArg`s
                Some(ast::GenericArg::TypeArg(make::type_arg(make::ext::ty_name(it.name()?))))
            }
        });

        make::generic_arg_list(args)
    }
}

pub trait Removable: AstNode {
    fn remove(&self);
}

impl Removable for ast::UseTree {
    fn remove(&self) {
        for dir in [Direction::Next, Direction::Prev] {
            if let Some(next_use_tree) = neighbor(self, dir) {
                let separators = self
                    .syntax()
                    .siblings_with_tokens(dir)
                    .skip(1)
                    .take_while(|it| it.as_node() != Some(next_use_tree.syntax()));
                ted::remove_all_iter(separators);
                break;
            }
        }
        ted::remove(self.syntax());
    }
}

impl ast::UseTree {
    /// Deletes the usetree node represented by the input. Recursively removes parents, including use nodes that become empty.
    pub fn remove_recursive(self) {
        let parent = self.syntax().parent();

        self.remove();

        if let Some(u) = parent.clone().and_then(ast::Use::cast) {
            if u.use_tree().is_none() {
                u.remove();
            }
        } else if let Some(u) = parent.and_then(ast::UseTreeList::cast) {
            if u.use_trees().next().is_none() {
                let parent = u.syntax().parent().and_then(ast::UseTree::cast);
                if let Some(u) = parent {
                    u.remove_recursive();
                }
            }
            u.remove_unnecessary_braces();
        }
    }

    pub fn get_or_create_use_tree_list(&self) -> ast::UseTreeList {
        match self.use_tree_list() {
            Some(it) => it,
            None => {
                let position = ted::Position::last_child_of(self.syntax());
                let use_tree_list = make::use_tree_list(empty()).clone_for_update();
                let mut elements = Vec::with_capacity(2);
                if self.coloncolon_token().is_none() {
                    elements.push(make::token(T![::]).into());
                }
                elements.push(use_tree_list.syntax().clone().into());
                ted::insert_all_raw(position, elements);
                use_tree_list
            }
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
    pub fn split_prefix(&self, prefix: &ast::Path) {
        debug_assert_eq!(self.path(), Some(prefix.top_path()));
        let path = self.path().unwrap();
        if &path == prefix && self.use_tree_list().is_none() {
            if self.star_token().is_some() {
                // path$0::* -> *
                if let Some(a) = self.coloncolon_token() {
                    ted::remove(a)
                }
                ted::remove(prefix.syntax());
            } else {
                // path$0 -> self
                let self_suffix =
                    make::path_unqualified(make::path_segment_self()).clone_for_update();
                ted::replace(path.syntax(), self_suffix.syntax());
            }
        } else if split_path_prefix(prefix).is_none() {
            return;
        }
        // At this point, prefix path is detached; _self_ use tree has suffix path.
        // Next, transform 'suffix' use tree into 'prefix::{suffix}'
        let subtree = self.clone_subtree().clone_for_update();
        ted::remove_all_iter(self.syntax().children_with_tokens());
        ted::insert(ted::Position::first_child_of(self.syntax()), prefix.syntax());
        self.get_or_create_use_tree_list().add_use_tree(subtree);

        fn split_path_prefix(prefix: &ast::Path) -> Option<()> {
            let parent = prefix.parent_path()?;
            let segment = parent.segment()?;
            if algo::has_errors(segment.syntax()) {
                return None;
            }
            for p in successors(parent.parent_path(), |it| it.parent_path()) {
                p.segment()?;
            }
            if let Some(a) = prefix.parent_path().and_then(|p| p.coloncolon_token()) {
                ted::remove(a)
            }
            ted::remove(prefix.syntax());
            Some(())
        }
    }

    /// Wraps the use tree in use tree list with no top level path (if it isn't already).
    ///
    /// # Examples
    ///
    /// `foo::bar` -> `{foo::bar}`
    ///
    /// `{foo::bar}` -> `{foo::bar}`
    pub fn wrap_in_tree_list(&self) -> Option<()> {
        if self.use_tree_list().is_some()
            && self.path().is_none()
            && self.star_token().is_none()
            && self.rename().is_none()
        {
            return None;
        }
        let subtree = self.clone_subtree().clone_for_update();
        ted::remove_all_iter(self.syntax().children_with_tokens());
        ted::append_child(
            self.syntax(),
            make::use_tree_list(once(subtree)).clone_for_update().syntax(),
        );
        Some(())
    }
}

impl ast::UseTreeList {
    pub fn add_use_tree(&self, use_tree: ast::UseTree) {
        let (position, elements) = match self.use_trees().last() {
            Some(last_tree) => (
                ted::Position::after(last_tree.syntax()),
                vec![
                    make::token(T![,]).into(),
                    make::tokens::single_space().into(),
                    use_tree.syntax.into(),
                ],
            ),
            None => {
                let position = match self.l_curly_token() {
                    Some(l_curly) => ted::Position::after(l_curly),
                    None => ted::Position::last_child_of(self.syntax()),
                };
                (position, vec![use_tree.syntax.into()])
            }
        };
        ted::insert_all_raw(position, elements);
    }
}

impl Removable for ast::Use {
    fn remove(&self) {
        let next_ws = self
            .syntax()
            .next_sibling_or_token()
            .and_then(|it| it.into_token())
            .and_then(ast::Whitespace::cast);
        if let Some(next_ws) = next_ws {
            let ws_text = next_ws.syntax().text();
            if let Some(rest) = ws_text.strip_prefix('\n') {
                if rest.is_empty() {
                    ted::remove(next_ws.syntax());
                } else {
                    ted::replace(next_ws.syntax(), make::tokens::whitespace(rest));
                }
            }
        }
        let prev_ws = self
            .syntax()
            .prev_sibling_or_token()
            .and_then(|it| it.into_token())
            .and_then(ast::Whitespace::cast);
        if let Some(prev_ws) = prev_ws {
            let ws_text = prev_ws.syntax().text();
            let prev_newline = ws_text.rfind('\n').map(|x| x + 1).unwrap_or(0);
            let rest = &ws_text[0..prev_newline];
            if rest.is_empty() {
                ted::remove(prev_ws.syntax());
            } else {
                ted::replace(prev_ws.syntax(), make::tokens::whitespace(rest));
            }
        }

        ted::remove(self.syntax());
    }
}

impl ast::Impl {
    pub fn get_or_create_assoc_item_list(&self) -> ast::AssocItemList {
        if self.assoc_item_list().is_none() {
            let assoc_item_list = make::assoc_item_list(None).clone_for_update();
            ted::append_child(self.syntax(), assoc_item_list.syntax());
        }
        self.assoc_item_list().unwrap()
    }
}

impl ast::AssocItemList {
    /// Adds a new associated item after all of the existing associated items.
    ///
    /// Attention! This function does align the first line of `item` with respect to `self`,
    /// but it does _not_ change indentation of other lines (if any).
    pub fn add_item(&self, editor: &SyntaxEditor, item: ast::AssocItem) {
        let make = editor.make();
        let (indent, position, whitespace) = match self.assoc_items().last() {
            Some(last_item) => (
                IndentLevel::from_node(last_item.syntax()),
                syntax_editor::Position::after(last_item.syntax()),
                "\n\n",
            ),
            None => match self.l_curly_token() {
                Some(l_curly) => {
                    normalize_ws_between_braces_with_editor(editor, self.syntax());
                    (
                        IndentLevel::from_token(&l_curly) + 1,
                        syntax_editor::Position::after(&l_curly),
                        "\n",
                    )
                }
                None => (
                    IndentLevel::zero(),
                    syntax_editor::Position::last_child_of(self.syntax()),
                    "\n",
                ),
            },
        };
        let elements: Vec<SyntaxElement> = vec![
            make.whitespace(&format!("{whitespace}{indent}")).into(),
            item.syntax().clone().into(),
        ];
        editor.insert_all(position, elements);
    }
}

impl ast::RecordExprFieldList {
    pub fn add_field(&self, field: ast::RecordExprField) {
        let is_multiline = self.syntax().text().contains_char('\n');
        let whitespace = if is_multiline {
            let indent = IndentLevel::from_node(self.syntax()) + 1;
            make::tokens::whitespace(&format!("\n{indent}"))
        } else {
            make::tokens::single_space()
        };

        if is_multiline {
            normalize_ws_between_braces(self.syntax());
        }

        let position = match self.fields().last() {
            Some(last_field) => {
                let comma = get_or_insert_comma_after(last_field.syntax());
                ted::Position::after(comma)
            }
            None => match self.l_curly_token() {
                Some(it) => ted::Position::after(it),
                None => ted::Position::last_child_of(self.syntax()),
            },
        };

        ted::insert_all(position, vec![whitespace.into(), field.syntax().clone().into()]);
        if is_multiline {
            ted::insert(ted::Position::after(field.syntax()), ast::make::token(T![,]));
        }
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
                .record_expr_field(editor.make().name_ref(&name_ref.text()), Some(expr));
            editor.replace(self.syntax(), new_field.syntax());
        }
    }
}

impl ast::RecordPatFieldList {
    pub fn add_field(&self, field: ast::RecordPatField) {
        let is_multiline = self.syntax().text().contains_char('\n');
        let whitespace = if is_multiline {
            let indent = IndentLevel::from_node(self.syntax()) + 1;
            make::tokens::whitespace(&format!("\n{indent}"))
        } else {
            make::tokens::single_space()
        };

        if is_multiline {
            normalize_ws_between_braces(self.syntax());
        }

        let position = match self.fields().last() {
            Some(last_field) => {
                let syntax = last_field.syntax();
                let comma = get_or_insert_comma_after(syntax);
                ted::Position::after(comma)
            }
            None => match self.l_curly_token() {
                Some(it) => ted::Position::after(it),
                None => ted::Position::last_child_of(self.syntax()),
            },
        };

        ted::insert_all(position, vec![whitespace.into(), field.syntax().clone().into()]);
        if is_multiline {
            ted::insert(ted::Position::after(field.syntax()), ast::make::token(T![,]));
        }
    }
}

fn get_or_insert_comma_after(syntax: &SyntaxNode) -> SyntaxToken {
    match syntax
        .siblings_with_tokens(Direction::Next)
        .filter_map(|it| it.into_token())
        .find(|it| it.kind() == T![,])
    {
        Some(it) => it,
        None => {
            let comma = ast::make::token(T![,]);
            ted::insert(ted::Position::after(syntax), &comma);
            comma
        }
    }
}

fn normalize_ws_between_braces(node: &SyntaxNode) -> Option<()> {
    let l = node
        .children_with_tokens()
        .filter_map(|it| it.into_token())
        .find(|it| it.kind() == T!['{'])?;
    let r = node
        .children_with_tokens()
        .filter_map(|it| it.into_token())
        .find(|it| it.kind() == T!['}'])?;

    let indent = IndentLevel::from_node(node);

    match l.next_sibling_or_token() {
        Some(ws)
            if ws.kind() == SyntaxKind::WHITESPACE
                && ws.next_sibling_or_token()?.into_token()? == r =>
        {
            ted::replace(ws, make::tokens::whitespace(&format!("\n{indent}")));
        }
        Some(ws) if ws.kind() == T!['}'] => {
            ted::insert(ted::Position::after(l), make::tokens::whitespace(&format!("\n{indent}")));
        }
        _ => (),
    }
    Some(())
}

fn normalize_ws_between_braces_with_editor(editor: &SyntaxEditor, node: &SyntaxNode) -> Option<()> {
    let make = editor.make();
    let l = node
        .children_with_tokens()
        .filter_map(|it| it.into_token())
        .find(|it| it.kind() == T!['{'])?;
    let r = node
        .children_with_tokens()
        .filter_map(|it| it.into_token())
        .find(|it| it.kind() == T!['}'])?;

    let indent = IndentLevel::from_node(node);

    match l.next_sibling_or_token() {
        Some(ws)
            if ws.kind() == SyntaxKind::WHITESPACE
                && ws.next_sibling_or_token()?.into_token()? == r =>
        {
            editor.replace(ws, make.whitespace(&format!("\n{indent}")));
        }
        Some(ws) if ws.kind() == T!['}'] => {
            editor
                .insert(syntax_editor::Position::after(l), make.whitespace(&format!("\n{indent}")));
        }
        _ => (),
    }
    Some(())
}

pub trait Indent: AstNode + Clone + Sized {
    fn indent_level(&self) -> IndentLevel {
        IndentLevel::from_node(self.syntax())
    }
    fn indent(&self, by: IndentLevel) {
        by.increase_indent(self.syntax());
    }
    fn dedent(&self, by: IndentLevel) {
        by.decrease_indent(self.syntax());
    }
    fn reindent_to(&self, target_level: IndentLevel) {
        let current_level = IndentLevel::from_node(self.syntax());
        self.dedent(current_level);
        self.indent(target_level);
    }
}

impl<N: AstNode + Clone> Indent for N {}

#[cfg(test)]
mod tests {
    use parser::Edition;

    use crate::SourceFile;

    use super::*;

    fn ast_mut_from_text<N: AstNode>(text: &str) -> N {
        let parse = SourceFile::parse(text, Edition::CURRENT);
        parse.tree().syntax().descendants().find_map(N::cast).unwrap().clone_for_update()
    }

    #[test]
    fn test_increase_indent() {
        let arm_list = ast_mut_from_text::<ast::Fn>(
            "fn foo() {
    ;
    ;
}",
        );
        arm_list.indent(IndentLevel(2));
        assert_eq!(
            arm_list.to_string(),
            "fn foo() {
            ;
            ;
        }",
        );
    }
}
