use ra_syntax::{ast, TextRange, AstNode, SyntaxNode, SyntaxKind::*};

pub fn ast_to_token_tree(ast: &ast::TokenTree) -> Option<(tt::Subtree, TokenMap)> {
    let mut token_map = TokenMap::default();
    let sub = convert_tt(ast.syntax(), &mut token_map)?;
    Some((sub, token_map))
}

#[derive(Default)]
pub struct TokenMap {
    tokens: Vec<TextRange>,
}

impl TokenMap {
    fn alloc(&mut self, range: TextRange) -> tt::TokenId {
        let id = self.tokens.len();
        let id = id as u32;
        self.tokens.push(range);
        tt::TokenId(id)
    }

    pub fn token_range(&self, id: tt::TokenId) -> Option<TextRange> {
        let id = id.0 as usize;
        self.tokens.get(id).map(|&it| it)
    }
}

fn convert_tt(tt: &SyntaxNode, token_map: &mut TokenMap) -> Option<tt::Subtree> {
    let first_child = tt.first_child()?;
    let last_child = tt.last_child()?;
    let delimiter = match (first_child.kind(), last_child.kind()) {
        (L_PAREN, R_PAREN) => tt::Delimiter::Parenthesis,
        (L_CURLY, R_CURLY) => tt::Delimiter::Brace,
        (L_BRACK, R_BRACK) => tt::Delimiter::Bracket,
        _ => return None,
    };
    let start_offset = tt.range().start();
    let mut token_trees = Vec::new();
    for child in tt.children().skip(1) {
        if child == first_child || child == last_child || child.kind().is_trivia() {
            continue;
        }
        if child.kind().is_punct() {
            let mut prev = None;
            for char in child.leaf_text().unwrap().chars() {
                if let Some(char) = prev {
                    token_trees.push(
                        tt::Leaf::from(tt::Punct {
                            char,
                            spacing: tt::Spacing::Joint,
                        })
                        .into(),
                    );
                }
                prev = Some(char)
            }
            if let Some(char) = prev {
                token_trees.push(
                    tt::Leaf::from(tt::Punct {
                        char,
                        spacing: tt::Spacing::Alone,
                    })
                    .into(),
                );
            }
        } else {
            let child: tt::TokenTree = if child.kind() == TOKEN_TREE {
                convert_tt(child, token_map)?.into()
            } else if child.kind().is_keyword() || child.kind() == IDENT {
                let text = child.leaf_text().unwrap().clone();
                let id = token_map.alloc(child.range() - start_offset);
                tt::Leaf::from(tt::Ident { text, id }).into()
            } else if child.kind().is_literal() {
                tt::Leaf::from(tt::Literal {
                    text: child.leaf_text().unwrap().clone(),
                })
                .into()
            } else {
                return None;
            };
            token_trees.push(child)
        }
    }

    let res = tt::Subtree {
        delimiter,
        token_trees,
    };
    Some(res)
}
