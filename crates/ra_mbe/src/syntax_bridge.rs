use ra_syntax::{ast, TextRange, TextUnit, AstNode, SyntaxNode, SyntaxKind::*, SourceFile, TreeArc};

#[derive(Debug, Default, PartialEq, Eq, Hash)]
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

#[derive(Debug, Default, PartialEq, Eq)]
pub struct RangesMap {
    // FIXME: account for cases where single input range maps to many output ranges.
    data: Vec<(TextRange, TextRange)>,
}

// FIXME: remove this impl
impl From<Vec<(TextRange, TextRange)>> for RangesMap {
    fn from(data: Vec<(TextRange, TextRange)>) -> RangesMap {
        RangesMap { data }
    }
}

impl RangesMap {
    /// Maps range in the source code to the range in the expanded code.
    pub fn map_forward(&self, src_range: TextRange) -> Option<TextRange> {
        for (s_range, t_range) in self.data.iter() {
            if src_range.is_subrange(&s_range) {
                let src_at_zero_range = src_range - src_range.start();
                let src_range_offset = src_range.start() - s_range.start();
                let src_range = src_at_zero_range + src_range_offset + t_range.start();
                return Some(src_range);
            }
        }
        None
    }
    /// Maps range in the expanded code to the range in the source code.
    pub fn map_back(&self, tgt_range: TextRange) -> Option<TextRange> {
        for (s_range, t_range) in self.data.iter() {
            if tgt_range.is_subrange(&t_range) {
                let tgt_at_zero_range = tgt_range - tgt_range.start();
                let tgt_range_offset = tgt_range.start() - t_range.start();
                let src_range = tgt_at_zero_range + tgt_range_offset + s_range.start();
                return Some(src_range);
            }
        }
        None
    }
}

pub fn ast_to_token_tree(ast: &ast::TokenTree) -> Option<(tt::Subtree, TokenMap)> {
    let mut token_map = TokenMap::default();
    let sub = convert_tt(ast.syntax(), &mut token_map)?;
    Some((sub, token_map))
}

pub fn parse_token_tree(
    tt: &tt::Subtree,
    token_map: &TokenMap,
) -> (TreeArc<SourceFile>, RangesMap) {
    let mut buf = String::new();
    let mut ranges_map = RangesMap::default();
    subtree_to_string(token_map, &mut buf, &mut ranges_map, tt);
    (SourceFile::parse(&buf), ranges_map)
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

fn subtree_to_string(
    token_map: &TokenMap,
    buf: &mut String,
    ranges_map: &mut RangesMap,
    subtree: &tt::Subtree,
) {
    let (l, r) = match subtree.delimiter {
        tt::Delimiter::Parenthesis => ("(", ")"),
        tt::Delimiter::Brace => ("{", "}"),
        tt::Delimiter::Bracket => ("[", "]"),
        tt::Delimiter::None => ("", ""),
    };
    buf.push_str(l);
    let mut needs_space = false;
    for tt in subtree.token_trees.iter() {
        if needs_space {
            buf.push_str(" ");
        }
        needs_space = true;
        match tt {
            tt::TokenTree::Leaf(leaf) => match leaf {
                tt::Leaf::Ident(i) => {
                    if let Some(src_range) = token_map.token_range(i.id) {
                        let tgt_range = TextRange::offset_len(
                            TextUnit::of_str(buf),
                            TextUnit::of_str(i.text.as_str()),
                        );
                        ranges_map.data.push((src_range, tgt_range));
                    }
                    buf.push_str(i.text.as_str());
                }
                tt::Leaf::Punct(p) => {
                    needs_space = p.spacing == tt::Spacing::Alone;
                    buf.push(p.char);
                }
                tt::Leaf::Literal(it) => buf.push_str(it.text.as_str()),
            },
            tt::TokenTree::Subtree(subtree) => {
                subtree_to_string(token_map, buf, ranges_map, subtree)
            }
        }
    }
    buf.push_str(r);
}
