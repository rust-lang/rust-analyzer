//! Structural editing for ast.

use std::iter::empty;

use parser::T;

use crate::{
    algo::neighbor,
    ast::{self, edit::AstNodeEdit, make, GenericParamsOwner, WhereClause},
    ted::{self, Position},
    AstNode, AstToken, Direction,
};

use super::NameOwner;

pub trait GenericParamsOwnerEdit: ast::GenericParamsOwner + AstNodeEdit {
    fn get_or_create_where_clause(&self) -> ast::WhereClause;
}

impl GenericParamsOwnerEdit for ast::Fn {
    fn get_or_create_where_clause(&self) -> WhereClause {
        if self.where_clause().is_none() {
            let position = if let Some(ty) = self.ret_type() {
                Position::after(ty.syntax())
            } else if let Some(param_list) = self.param_list() {
                Position::after(param_list.syntax())
            } else {
                Position::last_child_of(self.syntax())
            };
            create_where_clause(position)
        }
        self.where_clause().unwrap()
    }
}

impl GenericParamsOwnerEdit for ast::Impl {
    fn get_or_create_where_clause(&self) -> WhereClause {
        if self.where_clause().is_none() {
            let position = if let Some(items) = self.assoc_item_list() {
                Position::before(items.syntax())
            } else {
                Position::last_child_of(self.syntax())
            };
            create_where_clause(position)
        }
        self.where_clause().unwrap()
    }
}

impl GenericParamsOwnerEdit for ast::Trait {
    fn get_or_create_where_clause(&self) -> WhereClause {
        if self.where_clause().is_none() {
            let position = if let Some(items) = self.assoc_item_list() {
                Position::before(items.syntax())
            } else {
                Position::last_child_of(self.syntax())
            };
            create_where_clause(position)
        }
        self.where_clause().unwrap()
    }
}

impl GenericParamsOwnerEdit for ast::Struct {
    fn get_or_create_where_clause(&self) -> WhereClause {
        if self.where_clause().is_none() {
            let tfl = self.field_list().and_then(|fl| match fl {
                ast::FieldList::RecordFieldList(_) => None,
                ast::FieldList::TupleFieldList(it) => Some(it),
            });
            let position = if let Some(tfl) = tfl {
                Position::after(tfl.syntax())
            } else if let Some(gpl) = self.generic_param_list() {
                Position::after(gpl.syntax())
            } else if let Some(name) = self.name() {
                Position::after(name.syntax())
            } else {
                Position::last_child_of(self.syntax())
            };
            create_where_clause(position)
        }
        self.where_clause().unwrap()
    }
}

impl GenericParamsOwnerEdit for ast::Enum {
    fn get_or_create_where_clause(&self) -> WhereClause {
        if self.where_clause().is_none() {
            let position = if let Some(gpl) = self.generic_param_list() {
                Position::after(gpl.syntax())
            } else if let Some(name) = self.name() {
                Position::after(name.syntax())
            } else {
                Position::last_child_of(self.syntax())
            };
            create_where_clause(position)
        }
        self.where_clause().unwrap()
    }
}

fn create_where_clause(position: Position) {
    let where_clause = make::where_clause(empty()).clone_for_update();
    ted::insert(position, where_clause.syntax());
}

impl ast::WhereClause {
    pub fn add_predicate(&self, predicate: ast::WherePred) {
        if let Some(pred) = self.predicates().last() {
            if !pred.syntax().siblings_with_tokens(Direction::Next).any(|it| it.kind() == T![,]) {
                ted::append_child_raw(self.syntax(), make::token(T![,]));
            }
        }
        ted::append_child(self.syntax(), predicate.syntax())
    }
}

impl ast::TypeBoundList {
    pub fn remove(&self) {
        if let Some(colon) =
            self.syntax().siblings_with_tokens(Direction::Prev).find(|it| it.kind() == T![:])
        {
            ted::remove_all(colon..=self.syntax().clone().into())
        } else {
            ted::remove(self.syntax())
        }
    }
}

impl ast::UseTree {
    pub fn remove(&self) {
        for &dir in [Direction::Next, Direction::Prev].iter() {
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
        ted::remove(self.syntax())
    }
}

impl ast::Use {
    pub fn remove(&self) {
        let next_ws = self
            .syntax()
            .next_sibling_or_token()
            .and_then(|it| it.into_token())
            .and_then(ast::Whitespace::cast);
        if let Some(next_ws) = next_ws {
            let ws_text = next_ws.syntax().text();
            if let Some(rest) = ws_text.strip_prefix('\n') {
                if rest.is_empty() {
                    ted::remove(next_ws.syntax())
                } else {
                    ted::replace(next_ws.syntax(), make::tokens::whitespace(rest))
                }
            }
        }
        ted::remove(self.syntax())
    }
}

impl ast::Impl {
    /// Add a node to at the start of the impl block.
    pub fn push_front(&self, item: ast::AssocItem) {
        let l_curly = self.get_or_create_assoc_item_list().l_curly_token().unwrap();
        let position = Position::after(l_curly);
        ted::insert(position, item.syntax());
    }

    /// Add a node at the end of the impl block.
    pub fn push_back(&self, item: ast::AssocItem) {
        let r_curly = self.get_or_create_assoc_item_list().r_curly_token().unwrap();
        let position = Position::before(r_curly);
        ted::insert(position, item.syntax());
    }

    /// Get the associated item list of this impl, or create one if none exists.
    pub fn get_or_create_assoc_item_list(&self) -> ast::AssocItemList {
        self.assoc_item_list().unwrap_or_else(|| {
            let list = make::assoc_item_list();
            let position = Position::after(self.syntax());
            ted::insert(position, list.syntax());
            list
        })
    }
}
