//! Structural editing for ast using `SyntaxEditor`

use crate::{
    algo::neighbor,
    ast::{
        self, make, syntax_factory::SyntaxFactory, AstNode, Fn, GenericParam, HasGenericParams,
        HasName,
    },
    syntax_editor::{Position, SyntaxEditor},
    AstToken, Direction, SyntaxKind,
};

impl SyntaxEditor {
    /// Adds a new generic param to the function using `SyntaxEditor`
    pub fn add_generic_param(&mut self, function: &Fn, new_param: GenericParam) {
        match function.generic_param_list() {
            Some(generic_param_list) => match generic_param_list.generic_params().last() {
                Some(last_param) => {
                    // There exists a generic param list and it's not empty
                    let position = generic_param_list.r_angle_token().map_or_else(
                        || Position::last_child_of(function.syntax()),
                        Position::before,
                    );

                    if last_param
                        .syntax()
                        .next_sibling_or_token()
                        .map_or(false, |it| it.kind() == SyntaxKind::COMMA)
                    {
                        self.insert(
                            Position::after(last_param.syntax()),
                            new_param.syntax().clone(),
                        );
                        self.insert(
                            Position::after(last_param.syntax()),
                            make::token(SyntaxKind::WHITESPACE),
                        );
                        self.insert(
                            Position::after(last_param.syntax()),
                            make::token(SyntaxKind::COMMA),
                        );
                    } else {
                        let elements = vec![
                            make::token(SyntaxKind::COMMA).into(),
                            make::token(SyntaxKind::WHITESPACE).into(),
                            new_param.syntax().clone().into(),
                        ];
                        self.insert_all(position, elements);
                    }
                }
                None => {
                    // There exists a generic param list but it's empty
                    let position = Position::after(generic_param_list.l_angle_token().unwrap());
                    self.insert(position, new_param.syntax());
                }
            },
            None => {
                // There was no generic param list
                let position = if let Some(name) = function.name() {
                    Position::after(name.syntax)
                } else if let Some(fn_token) = function.fn_token() {
                    Position::after(fn_token)
                } else if let Some(param_list) = function.param_list() {
                    Position::before(param_list.syntax)
                } else {
                    Position::last_child_of(function.syntax())
                };
                let elements = vec![
                    make::token(SyntaxKind::L_ANGLE).into(),
                    new_param.syntax().clone().into(),
                    make::token(SyntaxKind::R_ANGLE).into(),
                ];
                self.insert_all(position, elements);
            }
        }
    }
}

pub trait Removable: AstNode {
    fn remove(&self, editor: &mut SyntaxEditor);
}

impl Removable for ast::Use {
    fn remove(&self, editor: &mut SyntaxEditor) {
        let make = SyntaxFactory::new();

        let next_ws = self
            .syntax()
            .next_sibling_or_token()
            .and_then(|it| it.into_token())
            .and_then(ast::Whitespace::cast);
        if let Some(next_ws) = next_ws {
            let ws_text = next_ws.syntax().text();
            if let Some(rest) = ws_text.strip_prefix('\n') {
                if rest.is_empty() {
                    editor.delete(next_ws.syntax());
                } else {
                    editor.replace(next_ws.syntax(), make.whitespace(rest));
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
                editor.delete(prev_ws.syntax());
            } else {
                editor.replace(prev_ws.syntax(), make.whitespace(rest));
            }
        }

        editor.delete(self.syntax());
    }
}

impl Removable for ast::UseTree {
    fn remove(&self, editor: &mut SyntaxEditor) {
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
