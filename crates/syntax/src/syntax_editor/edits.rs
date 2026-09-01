//! Structural editing for ast using `SyntaxEditor`

use crate::{
    Direction, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken, T,
    algo::neighbor,
    ast::{self, AstNode, HasGenericParams, HasName, edit::AstNodeEdit, edit::IndentLevel},
    syntax_editor::{Position, PositionRepr, SyntaxEditor},
    syntax_node::{clone_subtree_with_outer_trivia, make_token_with_trivia, token_text},
};

pub trait GetOrCreateWhereClause: ast::HasGenericParams {
    fn where_clause_position(&self) -> Option<Position>;

    fn get_or_create_where_clause(
        &self,
        editor: &SyntaxEditor,
        new_preds: impl Iterator<Item = ast::WherePred>,
    ) {
        let make = editor.make();
        let existing = self.where_clause();
        let all_preds: Vec<_> =
            existing.iter().flat_map(|wc| wc.predicates()).chain(new_preds).collect();
        let new_where = make.where_clause(all_preds);

        if let Some(existing) = &existing {
            editor.replace(existing.syntax(), new_where.syntax());
        } else if let Some(pos) = self.where_clause_position() {
            let anchor = match &pos.repr {
                PositionRepr::After(SyntaxElement::Node(node)) => node.last_token(),
                PositionRepr::After(SyntaxElement::Token(token)) => Some(token.clone()),
                PositionRepr::Before(SyntaxElement::Node(node)) => {
                    node.first_token().and_then(|it| it.prev_token())
                }
                PositionRepr::Before(SyntaxElement::Token(token)) => token.prev_token(),
                PositionRepr::FirstChild(_) => None,
            };
            let trailing: String = anchor
                .iter()
                .flat_map(|token| token.trailing_trivia())
                .map(|piece| piece.text().to_owned())
                .collect();
            if let Some(anchor) = anchor.filter(|_| !trailing.is_empty()) {
                let leading: String =
                    anchor.leading_trivia().map(|piece| piece.text().to_owned()).collect();
                let trimmed = make.token_with_trivia(anchor.kind(), anchor.text(), &leading, "");
                editor.replace_discard_trivia(anchor, trimmed);
            }
            let new_where =
                new_where.with_leading_trivia(" ", make).with_trailing_trivia(&trailing, make);
            editor.insert(pos, new_where.syntax().clone());
        }
    }
}

impl GetOrCreateWhereClause for ast::Fn {
    fn where_clause_position(&self) -> Option<Position> {
        if let Some(ty) = self.ret_type() {
            Some(Position::after(ty.syntax()))
        } else if let Some(param_list) = self.param_list() {
            Some(Position::after(param_list.syntax()))
        } else {
            Some(Position::last_child_of(self.syntax()))
        }
    }
}

impl GetOrCreateWhereClause for ast::Impl {
    fn where_clause_position(&self) -> Option<Position> {
        if let Some(ty) = self.self_ty() {
            Some(Position::after(ty.syntax()))
        } else {
            Some(Position::last_child_of(self.syntax()))
        }
    }
}

impl GetOrCreateWhereClause for ast::Trait {
    fn where_clause_position(&self) -> Option<Position> {
        if let Some(gpl) = self.generic_param_list() {
            Some(Position::after(gpl.syntax()))
        } else if let Some(name) = self.name() {
            Some(Position::after(name.syntax()))
        } else {
            Some(Position::last_child_of(self.syntax()))
        }
    }
}

impl GetOrCreateWhereClause for ast::TypeAlias {
    fn where_clause_position(&self) -> Option<Position> {
        if let Some(gpl) = self.generic_param_list() {
            Some(Position::after(gpl.syntax()))
        } else if let Some(name) = self.name() {
            Some(Position::after(name.syntax()))
        } else {
            Some(Position::last_child_of(self.syntax()))
        }
    }
}

impl GetOrCreateWhereClause for ast::Struct {
    fn where_clause_position(&self) -> Option<Position> {
        let tfl = self.field_list().and_then(|fl| match fl {
            ast::FieldList::RecordFieldList(_) => None,
            ast::FieldList::TupleFieldList(it) => Some(it),
        });
        if let Some(tfl) = tfl {
            Some(Position::after(tfl.syntax()))
        } else if let Some(gpl) = self.generic_param_list() {
            Some(Position::after(gpl.syntax()))
        } else if let Some(name) = self.name() {
            Some(Position::after(name.syntax()))
        } else {
            Some(Position::last_child_of(self.syntax()))
        }
    }
}

impl GetOrCreateWhereClause for ast::Enum {
    fn where_clause_position(&self) -> Option<Position> {
        if let Some(gpl) = self.generic_param_list() {
            Some(Position::after(gpl.syntax()))
        } else if let Some(name) = self.name() {
            Some(Position::after(name.syntax()))
        } else {
            Some(Position::last_child_of(self.syntax()))
        }
    }
}

impl SyntaxEditor {
    /// Adds a new generic param to the node using `SyntaxEditor`
    pub fn add_generic_param(
        &self,
        node: &impl ast::HasGenericParams,
        new_param: ast::GenericParam,
    ) {
        let make = self.make();
        match node.generic_param_list() {
            Some(generic_param_list) => {
                let is_lifetime = matches!(new_param, ast::GenericParam::LifetimeParam(_));

                if let Some(first_param) = generic_param_list.generic_params().next() {
                    let last_lifetime = generic_param_list
                        .generic_params()
                        .filter(|p| matches!(p, ast::GenericParam::LifetimeParam(_)))
                        .last();

                    if is_lifetime {
                        if let Some(last_lt) = last_lifetime {
                            let elements = vec![
                                make.token_trivia(SyntaxKind::COMMA, "", " ").into(),
                                new_param.syntax().clone().into(),
                            ];
                            self.insert_all(Position::after(last_lt.syntax()), elements);
                        } else {
                            // Insert before the first parameter
                            let elements = vec![
                                new_param.syntax().clone().into(),
                                make.token_trivia(SyntaxKind::COMMA, "", " ").into(),
                            ];
                            self.insert_all(Position::before(first_param.syntax()), elements);
                        }
                    } else {
                        let last_param = generic_param_list.generic_params().last().unwrap();
                        let elements = vec![
                            make.token_trivia(SyntaxKind::COMMA, "", " ").into(),
                            new_param.syntax().clone().into(),
                        ];
                        self.insert_all(Position::after(last_param.syntax()), elements);
                    }
                } else {
                    if let Some(l_angle) = generic_param_list.l_angle_token() {
                        self.insert(Position::after(l_angle), new_param.syntax().clone());
                    }
                }
            }
            None => {
                let position =
                    if let Some(name) = node.syntax().children().find_map(ast::Name::cast) {
                        Position::after(name.syntax())
                    } else if let Some(impl_node) = ast::Impl::cast(node.syntax().clone()) {
                        impl_node
                            .impl_token()
                            .map_or_else(|| Position::last_child_of(node.syntax()), Position::after)
                    } else if let Some(fn_node) = ast::Fn::cast(node.syntax().clone()) {
                        if let Some(fn_token) = fn_node.fn_token() {
                            Position::after(fn_token)
                        } else if let Some(param_list) = fn_node.param_list() {
                            Position::before(param_list.syntax())
                        } else {
                            Position::last_child_of(node.syntax())
                        }
                    } else {
                        Position::last_child_of(node.syntax())
                    };

                let elements = vec![
                    make.token(SyntaxKind::L_ANGLE).into(),
                    new_param.syntax().clone().into(),
                    make.token(SyntaxKind::R_ANGLE).into(),
                ];
                self.insert_all(position, elements);
            }
        }
    }
}

fn get_or_insert_comma_after(editor: &SyntaxEditor, syntax: &SyntaxNode) -> SyntaxToken {
    let make = editor.make();
    match comma_after(syntax) {
        Some(it) => it,
        None => {
            let comma = make.token(T![,]);
            editor.insert(Position::after(syntax), &comma);
            comma
        }
    }
}

impl ast::AssocItemList {
    /// Adds a new associated item after all of the existing associated items.
    ///
    /// Attention! This function does align the first line of `item` with respect to `self`,
    /// but it does _not_ change indentation of other lines (if any).
    pub fn add_items(
        &self,
        editor: &SyntaxEditor,
        items: Vec<ast::AssocItem>,
    ) -> Vec<ast::AssocItem> {
        let make = editor.make();
        let empty_braces = self
            .assoc_items()
            .next()
            .is_none()
            .then(|| self.l_curly_token().zip(self.r_curly_token()))
            .flatten()
            .filter(|(l_curly, r_curly)| {
                l_curly.trailing_trivia().chain(r_curly.leading_trivia()).all(|piece| {
                    matches!(piece.kind(), SyntaxKind::WHITESPACE | SyntaxKind::NEWLINE)
                })
            });

        let (indent, position, whitespace, item_trailing) =
            match (&empty_braces, self.assoc_items().last()) {
                (_, Some(last_item)) => (
                    IndentLevel::from_node(last_item.syntax()),
                    Position::after(last_item.syntax()),
                    "\n\n",
                    "",
                ),
                (Some((l_curly, _)), None) => {
                    (IndentLevel::from_token(l_curly) + 1, Position::after(l_curly), "\n", "")
                }
                (None, None) => match self.r_curly_token() {
                    Some(r_curly) => {
                        let on_own_line = self.l_curly_token().is_some_and(|it| {
                            it.trailing_trivia().any(|p| p.text().contains('\n'))
                        });
                        (
                            IndentLevel::from_node(self.syntax()) + 1,
                            Position::before(r_curly),
                            if on_own_line { "" } else { "\n" },
                            "\n",
                        )
                    }
                    None => (IndentLevel::zero(), Position::last_child_of(self.syntax()), "\n", ""),
                },
            };

        let items: Vec<ast::AssocItem> = items
            .into_iter()
            .enumerate()
            .map(|(i, item)| {
                let whitespace = if i != 0 { "\n\n" } else { whitespace };
                item.with_leading_trivia(&format!("{whitespace}{indent}"), make)
                    .with_trailing_trivia(item_trailing, make)
            })
            .collect();

        match empty_braces {
            Some((l_curly, r_curly)) => {
                let outer = IndentLevel::from_token(&l_curly);
                let before: String =
                    l_curly.leading_trivia().map(|it| it.text().to_owned()).collect();
                let mut elements: Vec<SyntaxElement> =
                    vec![make_token_with_trivia(T!['{'], "{", &before, "").into()];
                elements.extend(items.iter().map(|it| it.syntax().clone().into()));
                elements
                    .push(make_token_with_trivia(T!['}'], "}", &format!("\n{outer}"), "").into());
                editor.replace_all(l_curly.into()..=r_curly.into(), elements);
            }
            None => editor.insert_all(
                position,
                items.iter().map(|it| it.syntax().clone().into()).collect::<Vec<SyntaxElement>>(),
            ),
        }
        items
    }
}

impl ast::RecordExprFieldList {
    pub fn add_fields(
        &self,
        editor: &SyntaxEditor,
        fields: impl IntoIterator<Item = ast::RecordExprField>,
    ) {
        add_record_fields(
            editor,
            self.syntax(),
            self.fields().last().map(|it| it.syntax().clone()),
            self.l_curly_token(),
            fields.into_iter().map(|it| it.syntax().clone().into()),
        );
    }
}

impl ast::RecordPatFieldList {
    pub fn add_fields(
        &self,
        editor: &SyntaxEditor,
        fields: impl IntoIterator<Item = ast::RecordPatField>,
    ) {
        add_record_fields(
            editor,
            self.syntax(),
            self.fields().last().map(|it| it.syntax().clone()),
            self.l_curly_token(),
            fields.into_iter().map(|it| it.syntax().clone().into()),
        );
    }
}

fn add_record_fields(
    editor: &SyntaxEditor,
    field_list: &SyntaxNode,
    last_field: Option<SyntaxNode>,
    l_curly: Option<SyntaxToken>,
    fields: impl Iterator<Item = SyntaxElement>,
) {
    let fields = fields.collect::<Vec<_>>();
    if fields.is_empty() {
        return;
    }

    let make = editor.make();
    let is_multiline = token_text(field_list).contains('\n');
    let whitespace = if is_multiline {
        format!("\n{}", IndentLevel::from_node(field_list) + 1)
    } else {
        " ".to_owned()
    };
    let with_trivia =
        |element: SyntaxElement, leading: Option<&str>, trailing: Option<&str>| match element {
            SyntaxElement::Node(node) => {
                SyntaxElement::Node(clone_subtree_with_outer_trivia(&node, leading, trailing))
            }
            SyntaxElement::Token(token) => {
                let pieces = |it: SyntaxToken| it.text().to_owned();
                let leading = leading
                    .map_or_else(|| token.leading_trivia().map(pieces).collect(), str::to_owned);
                let trailing = trailing
                    .map_or_else(|| token.trailing_trivia().map(pieces).collect(), str::to_owned);
                SyntaxElement::Token(make_token_with_trivia(
                    token.kind(),
                    token.text(),
                    &leading,
                    &trailing,
                ))
            }
        };

    if is_multiline {
        normalize_ws_between_braces(editor, field_list);
    }

    let skip_leading = !is_multiline
        && last_field.is_none()
        && l_curly.as_ref().is_some_and(|it| it.trailing_trivia().next().is_some());

    let mut elements = Vec::new();
    let next_after_insert;
    let position = match last_field {
        Some(last_field) => match comma_after(&last_field) {
            Some(comma) => {
                next_after_insert = comma.next_sibling_or_token();
                Position::after(comma)
            }
            None => {
                next_after_insert = last_field.next_sibling_or_token();
                elements.push(make.token(T![,]).into());
                Position::after(last_field)
            }
        },
        None => match l_curly {
            Some(it) => {
                next_after_insert = it.next_sibling_or_token();
                Position::after(it)
            }
            None => {
                next_after_insert = None;
                Position::last_child_of(field_list)
            }
        },
    };

    let fields_len = fields.len();
    for (idx, field) in fields.into_iter().enumerate() {
        let leading = if idx == 0 && skip_leading { "" } else { whitespace.as_str() };
        elements.push(with_trivia(field, Some(leading), None));
        if is_multiline || idx + 1 != fields_len {
            elements.push(make.token(T![,]).into());
        }
    }
    if !is_multiline && next_after_insert.is_some() {
        let last = elements.pop().expect("fields is not empty");
        elements.push(with_trivia(last, None, Some(" ")));
    }

    editor.insert_all(position, elements);
}

fn comma_after(syntax: &SyntaxNode) -> Option<SyntaxToken> {
    syntax
        .siblings_with_tokens(Direction::Next)
        .filter_map(|it| it.into_token())
        .find(|it| it.kind() == T![,])
}

impl ast::Impl {
    pub fn get_or_create_assoc_item_list_with_editor(
        &self,
        editor: &SyntaxEditor,
    ) -> ast::AssocItemList {
        let make = editor.make();
        if let Some(list) = self.assoc_item_list() {
            list
        } else {
            let list = make.assoc_item_list_empty().with_leading_trivia(" ", make);
            editor.insert(Position::last_child_of(self.syntax()), list.syntax().clone());
            list
        }
    }
}

impl ast::VariantList {
    pub fn add_variant(&self, editor: &SyntaxEditor, variant: &ast::Variant) {
        let make = editor.make();
        let (indent, position) = match self.variants().last() {
            Some(last_item) => (
                IndentLevel::from_node(last_item.syntax()),
                Position::after(get_or_insert_comma_after(editor, last_item.syntax())),
            ),
            None => match self.l_curly_token() {
                Some(l_curly) => {
                    normalize_ws_between_braces(editor, self.syntax());
                    (IndentLevel::from_token(&l_curly) + 1, Position::after(&l_curly))
                }
                None => (IndentLevel::zero(), Position::last_child_of(self.syntax())),
            },
        };
        let variant = variant.with_leading_trivia(&format!("{}{indent}", "\n"), make);
        let elements: Vec<SyntaxElement> =
            vec![variant.syntax().clone().into(), make.token(T![,]).into()];
        editor.insert_all(position, elements);
    }
}

impl ast::Fn {
    pub fn replace_or_insert_body(&self, editor: &SyntaxEditor, body: ast::BlockExpr) {
        let make = editor.make();
        if let Some(old_body) = self.body() {
            editor.replace(old_body.syntax(), body.syntax());
        } else {
            let body = body.with_leading_trivia(" ", make);
            if let Some(semicolon) = self.semicolon_token() {
                editor.replace_discard_trivia(semicolon, body.syntax().clone());
            } else {
                editor.insert(Position::last_child_of(self.syntax()), body.syntax().clone());
            }
        }
    }
}

fn normalize_ws_between_braces(editor: &SyntaxEditor, node: &SyntaxNode) -> Option<()> {
    let l = node
        .children_with_tokens()
        .filter_map(|it| it.into_token())
        .find(|it| it.kind() == T!['{'])?;
    let r = node
        .children_with_tokens()
        .filter_map(|it| it.into_token())
        .find(|it| it.kind() == T!['}'])?;

    let indent = IndentLevel::from_node(node);

    let interior_is_blank = l
        .trailing_trivia()
        .chain(r.leading_trivia())
        .all(|piece| matches!(piece.kind(), SyntaxKind::WHITESPACE | SyntaxKind::NEWLINE));
    if l.next_sibling_or_token()?.into_token()? == r && interior_is_blank {
        let before: String = l.leading_trivia().map(|it| it.text().to_owned()).collect();
        editor.replace_discard_trivia(l.clone(), make_token_with_trivia(T!['{'], "{", &before, ""));
        let after: String = r.trailing_trivia().map(|it| it.text().to_owned()).collect();
        editor.replace_discard_trivia(
            r,
            make_token_with_trivia(T!['}'], "}", &format!("\n{indent}"), &after),
        );
    }
    Some(())
}

pub trait Removable: AstNode {
    fn remove(&self, editor: &SyntaxEditor);
}

impl Removable for ast::TypeBoundList {
    fn remove(&self, editor: &SyntaxEditor) {
        match self.syntax().siblings_with_tokens(Direction::Prev).find(|it| it.kind() == T![:]) {
            Some(colon) => editor.delete_all(colon..=self.syntax().clone().into()),
            None => editor.delete(self.syntax()),
        }
    }
}

impl Removable for ast::Use {
    fn remove(&self, editor: &SyntaxEditor) {
        editor.delete(self.syntax());
    }
}

impl Removable for ast::UseTree {
    fn remove(&self, editor: &SyntaxEditor) {
        for dir in [Direction::Next, Direction::Prev] {
            if let Some(next_use_tree) = neighbor(self, dir) {
                let separators = self
                    .syntax()
                    .siblings_with_tokens(dir)
                    .skip(1)
                    .take_while(|it| it.as_node() != Some(next_use_tree.syntax()));
                for sep in separators {
                    editor.delete(sep);
                }
                break;
            }
        }
        editor.delete(self.syntax());
    }
}

#[cfg(test)]
mod tests {
    use parser::Edition;
    use stdx::trim_indent;
    use test_utils::assert_eq_text;

    use crate::{SourceFile, ast::syntax_factory::SyntaxFactory};

    use super::*;

    fn ast_from_text<N: AstNode>(text: &str) -> N {
        let parse = SourceFile::parse(text, Edition::CURRENT);
        let node = match parse.tree().syntax().descendants().find_map(N::cast) {
            Some(it) => it,
            None => {
                let node = std::any::type_name::<N>();
                panic!("Failed to make ast node `{node}` from text `{text}`")
            }
        };
        let node = node.clone_subtree();
        assert_eq!(node.syntax().text_range().start(), 0.into());
        node
    }

    #[test]
    fn add_variant_to_empty_enum() {
        let make = SyntaxFactory::without_mappings();
        let variant = make.variant(None, make.name("Bar"), None, None);

        check_add_variant(
            r#"
enum Foo {}
"#,
            r#"
enum Foo {
    Bar,
}
"#,
            variant,
        );
    }

    #[test]
    fn add_variant_to_non_empty_enum() {
        let make = SyntaxFactory::without_mappings();
        let variant = make.variant(None, make.name("Baz"), None, None);

        check_add_variant(
            r#"
enum Foo {
    Bar,
}
"#,
            r#"
enum Foo {
    Bar,
    Baz,
}
"#,
            variant,
        );
    }

    #[test]
    fn add_variant_with_tuple_field_list() {
        let make = SyntaxFactory::without_mappings();
        let variant = make.variant(
            None,
            make.name("Baz"),
            Some(make.tuple_field_list([make.tuple_field(None, make.ty("bool"))]).into()),
            None,
        );

        check_add_variant(
            r#"
enum Foo {
    Bar,
}
"#,
            r#"
enum Foo {
    Bar,
    Baz(bool),
}
"#,
            variant,
        );
    }

    #[test]
    fn add_variant_with_record_field_list() {
        let make = SyntaxFactory::without_mappings();
        let variant = make.variant(
            None,
            make.name("Baz"),
            Some(
                make.record_field_list([make.record_field(None, make.name("x"), make.ty("bool"))])
                    .into(),
            ),
            None,
        );

        check_add_variant(
            r#"
enum Foo {
    Bar,
}
"#,
            r#"
enum Foo {
    Bar,
    Baz { x: bool },
}
"#,
            variant,
        );
    }

    fn check_add_variant(before: &str, expected: &str, variant: ast::Variant) {
        let (editor, enum_) = SyntaxEditor::with_ast_node(&ast_from_text::<ast::Enum>(before));
        if let Some(it) = enum_.variant_list() {
            it.add_variant(&editor, &variant)
        }
        let edit = editor.finish();
        let after = edit.new_root.to_string();
        assert_eq_text!(&trim_indent(expected.trim()), &trim_indent(after.trim()));
    }
}
