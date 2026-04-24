//! Semantic Tokens helpers

use std::{fmt, ops};

use lsp_types::{Range, SemanticTokenModifiers, SemanticTokens, SemanticTokensEdit};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[repr(u32)]
#[derive(EnumIter, Debug, PartialEq, Clone, Copy)]
pub(crate) enum SupportedType {
    Comment,
    Decorator,
    EnumMember,
    Enum,
    Function,
    Interface,
    Keyword,
    Macro,
    Method,
    Namespace,
    Number,
    Operator,
    Parameter,
    Property,
    String,
    Struct,
    TypeParameter,
    Variable,
    Type,
    Label,
    Angle,
    Arithmetic,
    AttributeBracket,
    Attribute,
    Bitwise,
    Boolean,
    Brace,
    Bracket,
    BuiltinAttribute,
    BuiltinType,
    Char,
    Colon,
    Comma,
    Comparison,
    ConstParameter,
    Const,
    DeriveHelper,
    Derive,
    Dot,
    EscapeSequence,
    FormatSpecifier,
    Generic,
    InvalidEscapeSequence,
    Lifetime,
    Logical,
    MacroBang,
    Negation,
    Parenthesis,
    ProcMacro,
    Punctuation,
    SelfKeyword,
    SelfTypeKeyword,
    Semicolon,
    Static,
    ToolModule,
    TypeAlias,
    Union,
    UnresolvedReference,
}

impl From<&SupportedType> for String {
    fn from(e: &SupportedType) -> Self {
        match *e {
            SupportedType::Comment => ::lsp_types::SemanticTokenTypes::Comment.into(),
            SupportedType::Decorator => ::lsp_types::SemanticTokenTypes::Decorator.into(),
            SupportedType::EnumMember => ::lsp_types::SemanticTokenTypes::EnumMember.into(),
            SupportedType::Enum => ::lsp_types::SemanticTokenTypes::Enum.into(),
            SupportedType::Function => ::lsp_types::SemanticTokenTypes::Function.into(),
            SupportedType::Interface => ::lsp_types::SemanticTokenTypes::Interface.into(),
            SupportedType::Keyword => ::lsp_types::SemanticTokenTypes::Keyword.into(),
            SupportedType::Macro => ::lsp_types::SemanticTokenTypes::Macro.into(),
            SupportedType::Method => ::lsp_types::SemanticTokenTypes::Method.into(),
            SupportedType::Namespace => ::lsp_types::SemanticTokenTypes::Namespace.into(),
            SupportedType::Number => ::lsp_types::SemanticTokenTypes::Number.into(),
            SupportedType::Operator => ::lsp_types::SemanticTokenTypes::Operator.into(),
            SupportedType::Parameter => ::lsp_types::SemanticTokenTypes::Parameter.into(),
            SupportedType::Property => ::lsp_types::SemanticTokenTypes::Property.into(),
            SupportedType::String => ::lsp_types::SemanticTokenTypes::String.into(),
            SupportedType::Struct => ::lsp_types::SemanticTokenTypes::Struct.into(),
            SupportedType::TypeParameter => ::lsp_types::SemanticTokenTypes::TypeParameter.into(),
            SupportedType::Variable => ::lsp_types::SemanticTokenTypes::Variable.into(),
            SupportedType::Type => ::lsp_types::SemanticTokenTypes::Type.into(),
            SupportedType::Label => ::lsp_types::SemanticTokenTypes::Label.into(),
            SupportedType::Angle => "angle".to_owned(),
            SupportedType::Arithmetic => "arithmetic".to_owned(),
            SupportedType::AttributeBracket => "attributeBracket".to_owned(),
            SupportedType::Attribute => "attribute".to_owned(),
            SupportedType::Bitwise => "bitwise".to_owned(),
            SupportedType::Boolean => "boolean".to_owned(),
            SupportedType::Brace => "brace".to_owned(),
            SupportedType::Bracket => "bracket".to_owned(),
            SupportedType::BuiltinAttribute => "builtinAttribute".to_owned(),
            SupportedType::BuiltinType => "builtinType".to_owned(),
            SupportedType::Char => "character".to_owned(),
            SupportedType::Colon => "colon".to_owned(),
            SupportedType::Comma => "comma".to_owned(),
            SupportedType::Comparison => "comparison".to_owned(),
            SupportedType::ConstParameter => "constParameter".to_owned(),
            SupportedType::Const => "const".to_owned(),
            SupportedType::DeriveHelper => "deriveHelper".to_owned(),
            SupportedType::Derive => "derive".to_owned(),
            SupportedType::Dot => "dot".to_owned(),
            SupportedType::EscapeSequence => "escapeSequence".to_owned(),
            SupportedType::FormatSpecifier => "formatSpecifier".to_owned(),
            SupportedType::Generic => "generic".to_owned(),
            SupportedType::InvalidEscapeSequence => "invalidEscapeSequence".to_owned(),
            SupportedType::Lifetime => "lifetime".to_owned(),
            SupportedType::Logical => "logical".to_owned(),
            SupportedType::MacroBang => "macroBang".to_owned(),
            SupportedType::Negation => "negation".to_owned(),
            SupportedType::Parenthesis => "parenthesis".to_owned(),
            SupportedType::ProcMacro => "procMacro".to_owned(),
            SupportedType::Punctuation => "punctuation".to_owned(),
            SupportedType::SelfKeyword => "selfKeyword".to_owned(),
            SupportedType::SelfTypeKeyword => "selfTypeKeyword".to_owned(),
            SupportedType::Semicolon => "semicolon".to_owned(),
            SupportedType::Static => "static".to_owned(),
            SupportedType::ToolModule => "toolModule".to_owned(),
            SupportedType::TypeAlias => "typeAlias".to_owned(),
            SupportedType::Union => "union".to_owned(),
            SupportedType::UnresolvedReference => "unresolvedReference".to_owned(),
        }
    }
}

impl fmt::Display for SupportedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string: String = self.into();
        write!(f, "{string}")
    }
}

pub(crate) fn standard_fallback_type(token: SupportedType) -> Option<SupportedType> {
    Some(match token {
        SupportedType::Comment => SupportedType::Comment,
        SupportedType::Decorator => SupportedType::Decorator,
        SupportedType::EnumMember => SupportedType::EnumMember,
        SupportedType::Enum => SupportedType::Enum,
        SupportedType::Function => SupportedType::Function,
        SupportedType::Interface => SupportedType::Interface,
        SupportedType::Keyword => SupportedType::Keyword,
        SupportedType::Macro => SupportedType::Macro,
        SupportedType::Method => SupportedType::Method,
        SupportedType::Namespace => SupportedType::Namespace,
        SupportedType::Number => SupportedType::Number,
        SupportedType::Operator => SupportedType::Operator,
        SupportedType::Parameter => SupportedType::Parameter,
        SupportedType::Property => SupportedType::Property,
        SupportedType::String => SupportedType::String,
        SupportedType::Struct => SupportedType::Struct,
        SupportedType::TypeParameter => SupportedType::TypeParameter,
        SupportedType::Variable => SupportedType::Variable,
        SupportedType::Type => SupportedType::Type,
        SupportedType::Label => SupportedType::Label,
        _ => return None,
    })
}

#[repr(u32)]
#[derive(EnumIter, Debug, PartialEq, Clone, Copy)]
pub(crate) enum SupportedModifiers {
    Async,
    Documentation,
    Declaration,
    Static,
    DefaultLibrary,
    Deprecated,
    Associated,
    AttributeModifier,
    Callable,
    Constant,
    Consuming,
    ControlFlow,
    CrateRoot,
    Injected,
    IntraDocLink,
    Library,
    MacroModifier,
    Mutable,
    ProcMacroModifier,
    Public,
    Reference,
    TraitModifier,
    Unsafe,
}

impl From<&SupportedModifiers> for String {
    fn from(e: &SupportedModifiers) -> Self {
        match *e {
            SupportedModifiers::Async => SemanticTokenModifiers::Async.into(),
            SupportedModifiers::Documentation => SemanticTokenModifiers::Documentation.into(),
            SupportedModifiers::Declaration => SemanticTokenModifiers::Declaration.into(),
            SupportedModifiers::Static => SemanticTokenModifiers::Static.into(),
            SupportedModifiers::DefaultLibrary => SemanticTokenModifiers::DefaultLibrary.into(),
            SupportedModifiers::Deprecated => SemanticTokenModifiers::Deprecated.into(),
            SupportedModifiers::Associated => "associated".to_owned(),
            SupportedModifiers::AttributeModifier => "attribute".to_owned(),
            SupportedModifiers::Callable => "callable".to_owned(),
            SupportedModifiers::Constant => "constant".to_owned(),
            SupportedModifiers::Consuming => "consuming".to_owned(),
            SupportedModifiers::ControlFlow => "controlFlow".to_owned(),
            SupportedModifiers::CrateRoot => "crateRoot".to_owned(),
            SupportedModifiers::Injected => "injected".to_owned(),
            SupportedModifiers::IntraDocLink => "intraDocLink".to_owned(),
            SupportedModifiers::Library => "library".to_owned(),
            SupportedModifiers::MacroModifier => "macro".to_owned(),
            SupportedModifiers::Mutable => "mutable".to_owned(),
            SupportedModifiers::ProcMacroModifier => "procMacro".to_owned(),
            SupportedModifiers::Public => "public".to_owned(),
            SupportedModifiers::Reference => "reference".to_owned(),
            SupportedModifiers::TraitModifier => "trait".to_owned(),
            SupportedModifiers::Unsafe => "unsafe".to_owned(),
        }
    }
}

impl fmt::Display for SupportedModifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string: String = self.into();
        write!(f, "{string}")
    }
}

const STANDARD_MOD: [SupportedModifiers; 6] = [
    SupportedModifiers::Async,
    SupportedModifiers::Documentation,
    SupportedModifiers::Declaration,
    SupportedModifiers::Static,
    SupportedModifiers::DefaultLibrary,
    SupportedModifiers::Deprecated,
];
const LAST_STANDARD_MOD: usize = STANDARD_MOD.len() - 1;

#[derive(Default)]
pub(crate) struct ModifierSet(pub(crate) u32);

impl ModifierSet {
    pub(crate) fn standard_fallback(&mut self) {
        // Remove all non standard modifiers
        self.0 &= !(!0u32 << LAST_STANDARD_MOD)
    }
}

impl ops::BitOrAssign<SupportedModifiers> for ModifierSet {
    fn bitor_assign(&mut self, rhs: SupportedModifiers) {
        let idx = SupportedModifiers::iter().position(|it| it == rhs).unwrap();
        self.0 |= 1 << idx;
    }
}

/// Tokens are encoded relative to each other.
///
/// This is a direct port of <https://github.com/microsoft/vscode-languageserver-node/blob/f425af9de46a0187adb78ec8a46b9b2ce80c5412/server/src/sematicTokens.proposed.ts#L45>
pub(crate) struct SemanticTokensBuilder {
    id: String,
    prev_line: u32,
    prev_char: u32,
    data: Vec<lsp_types::SemanticToken>,
}

impl SemanticTokensBuilder {
    pub(crate) fn new(id: String) -> Self {
        SemanticTokensBuilder { id, prev_line: 0, prev_char: 0, data: Vec::new() }
    }

    /// Push a new token onto the builder
    pub(crate) fn push(&mut self, range: Range, token_index: u32, modifier_bitset: u32) {
        let mut push_line = range.start.line;
        let mut push_char = range.start.character;

        if !self.data.is_empty() {
            push_line -= self.prev_line;
            if push_line == 0 {
                push_char -= self.prev_char;
            }
        }

        // A token cannot be multiline
        let token_len = range.end.character - range.start.character;

        let token = lsp_types::SemanticToken {
            delta_line: push_line,
            delta_start: push_char,
            length: token_len,
            token_type: token_index,
            token_modifiers_bitset: modifier_bitset,
        };

        self.data.push(token);

        self.prev_line = range.start.line;
        self.prev_char = range.start.character;
    }

    pub(crate) fn build(self) -> SemanticTokens {
        SemanticTokens { result_id: Some(self.id), data: self.data }
    }
}

pub(crate) fn diff_tokens(
    old: &[lsp_types::SemanticToken],
    new: &[lsp_types::SemanticToken],
) -> Vec<SemanticTokensEdit> {
    let offset = new.iter().zip(old.iter()).take_while(|&(n, p)| n == p).count();

    let (_, old) = old.split_at(offset);
    let (_, new) = new.split_at(offset);

    let offset_from_end =
        new.iter().rev().zip(old.iter().rev()).take_while(|&(n, p)| n == p).count();

    let (old, _) = old.split_at(old.len() - offset_from_end);
    let (new, _) = new.split_at(new.len() - offset_from_end);

    if old.is_empty() && new.is_empty() {
        vec![]
    } else {
        // The lsp data field is actually a byte-diff but we
        // travel in tokens so `start` and `delete_count` are in multiples of the
        // serialized size of `SemanticToken`.
        vec![SemanticTokensEdit {
            start: 5 * offset as u32,
            delete_count: 5 * old.len() as u32,
            data: Some(new.into()),
        }]
    }
}

pub(crate) fn type_index(kind: SupportedType) -> u32 {
    kind as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from(t: (u32, u32, u32, u32, u32)) -> lsp_types::SemanticToken {
        lsp_types::SemanticToken {
            delta_line: t.0,
            delta_start: t.1,
            length: t.2,
            token_type: t.3,
            token_modifiers_bitset: t.4,
        }
    }

    #[test]
    fn test_diff_insert_at_end() {
        let before = [from((1, 2, 3, 4, 5)), from((6, 7, 8, 9, 10))];
        let after = [from((1, 2, 3, 4, 5)), from((6, 7, 8, 9, 10)), from((11, 12, 13, 14, 15))];

        let edits = diff_tokens(&before, &after);
        assert_eq!(
            edits[0],
            SemanticTokensEdit {
                start: 10,
                delete_count: 0,
                data: Some(vec![from((11, 12, 13, 14, 15))])
            }
        );
    }

    #[test]
    fn test_diff_insert_at_beginning() {
        let before = [from((1, 2, 3, 4, 5)), from((6, 7, 8, 9, 10))];
        let after = [from((11, 12, 13, 14, 15)), from((1, 2, 3, 4, 5)), from((6, 7, 8, 9, 10))];

        let edits = diff_tokens(&before, &after);
        assert_eq!(
            edits[0],
            SemanticTokensEdit {
                start: 0,
                delete_count: 0,
                data: Some(vec![from((11, 12, 13, 14, 15))])
            }
        );
    }

    #[test]
    fn test_diff_insert_in_middle() {
        let before = [from((1, 2, 3, 4, 5)), from((6, 7, 8, 9, 10))];
        let after = [
            from((1, 2, 3, 4, 5)),
            from((10, 20, 30, 40, 50)),
            from((60, 70, 80, 90, 100)),
            from((6, 7, 8, 9, 10)),
        ];

        let edits = diff_tokens(&before, &after);
        assert_eq!(
            edits[0],
            SemanticTokensEdit {
                start: 5,
                delete_count: 0,
                data: Some(vec![from((10, 20, 30, 40, 50)), from((60, 70, 80, 90, 100))])
            }
        );
    }

    #[test]
    fn test_diff_remove_from_end() {
        let before = [from((1, 2, 3, 4, 5)), from((6, 7, 8, 9, 10)), from((11, 12, 13, 14, 15))];
        let after = [from((1, 2, 3, 4, 5)), from((6, 7, 8, 9, 10))];

        let edits = diff_tokens(&before, &after);
        assert_eq!(edits[0], SemanticTokensEdit { start: 10, delete_count: 5, data: Some(vec![]) });
    }

    #[test]
    fn test_diff_remove_from_beginning() {
        let before = [from((11, 12, 13, 14, 15)), from((1, 2, 3, 4, 5)), from((6, 7, 8, 9, 10))];
        let after = [from((1, 2, 3, 4, 5)), from((6, 7, 8, 9, 10))];

        let edits = diff_tokens(&before, &after);
        assert_eq!(edits[0], SemanticTokensEdit { start: 0, delete_count: 5, data: Some(vec![]) });
    }

    #[test]
    fn test_diff_remove_from_middle() {
        let before = [
            from((1, 2, 3, 4, 5)),
            from((10, 20, 30, 40, 50)),
            from((60, 70, 80, 90, 100)),
            from((6, 7, 8, 9, 10)),
        ];
        let after = [from((1, 2, 3, 4, 5)), from((6, 7, 8, 9, 10))];

        let edits = diff_tokens(&before, &after);
        assert_eq!(edits[0], SemanticTokensEdit { start: 5, delete_count: 10, data: Some(vec![]) });
    }
}
