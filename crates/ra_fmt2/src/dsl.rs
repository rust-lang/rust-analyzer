use crate::{pattern::{Pattern, PatternSet}};
use ra_syntax::{SyntaxElement, SyntaxKind::*,};
use std::iter::successors;

/// `SpacingRule` describes whitespace requirements between `SyntaxElement` Note
/// that it doesn't handle indentation (first whitespace on a line), there's
/// `IndentRule` for that!
#[derive(Debug)]
pub(crate) struct SpacingRule {
    /// An element to which this spacing rule applies
    pub(crate) pattern: Pattern,
    /// How much space to add/remove at the start or end of the element.
    pub(crate) space: Space,
}

/// Make `SpacingRule` usable with `PatternSet`
impl AsRef<Pattern> for SpacingRule {
    fn as_ref(&self) -> &Pattern {
        &self.pattern
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Space {
    /// How much space to add.
    pub(crate) value: SpaceValue,
    /// Should the space be added before, after or around the element?
    pub(crate) loc: SpaceLoc,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SpaceValue {
    /// Single whitespace char, like ` `
    Single,
    /// Single whitespace char, like ` `, but preserve existing line break.
    SingleOptionalNewline,
    /// A single newline (`\n`) char
    Newline,
    /// No whitespace at all.
    None,
    /// No space, but preserve existing line break.
    NoneOptionalNewline,
    /// If the parent element fits into a single line, a single space.
    /// Otherwise, at least one newline.
    /// Existing newlines are preserved.
    SingleOrNewline,
    /// If the parent element fits into a single line, no space.
    /// Otherwise, at least one newline.
    /// Existing newlines are preserved.
    NoneOrNewline,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SpaceLoc {
    /// Before the element.
    Before,
    /// After the element.
    After,
    /// On the both sides of the element.
    Around,
}

/// A builder to conveniently specify a set of `SpacingRule`s
#[derive(Debug, Default)]
pub(crate) struct SpacingDsl {
    pub(crate) rules: Vec<SpacingRule>,
    #[cfg(test)]
    pub(crate) tests: Vec<(&'static str, &'static str)>,
}

impl SpacingDsl {
    pub(crate) fn rule(&mut self, rule: SpacingRule) -> &mut Self {
        self.rules.push(rule);
        self
    }

    pub(crate) fn inside(&mut self, parent: impl Into<Pattern>) -> SpacingRuleBuilder {
        SpacingRuleBuilder {
            dsl: self,
            parent: parent.into(),
            child: None,
            between: None,
            loc: None,
        }
    }

    pub(crate) fn test(&mut self, before: &'static str, after: &'static str) -> &mut Self {
        #[cfg(test)]
        {
            self.tests.push((before, after));
        }
        let _ = (before, after);
        self
    }
}
    /// A builder to conveniently specify a single rule.
pub(crate) struct SpacingRuleBuilder<'a> {
    dsl: &'a mut SpacingDsl,
    parent: Pattern,
    child: Option<Pattern>,
    between: Option<(Pattern, Pattern)>,
    loc: Option<SpaceLoc>,
}

impl<'a> SpacingRuleBuilder<'a> {
    /// The rule applies to both sides of the element `child`.
    pub(crate) fn around(mut self, child: impl Into<Pattern>) -> SpacingRuleBuilder<'a> {
        self.child = Some(child.into());
        self.loc = Some(SpaceLoc::Around);
        self
    }
    /// The rule applies to the leading whitespace before `child`.
    pub(crate) fn before(mut self, child: impl Into<Pattern>) -> SpacingRuleBuilder<'a> {
        self.child = Some(child.into());
        self.loc = Some(SpaceLoc::Before);
        self
    }
    /// The rule applies to the trailing whitespace after `child`.
    pub(crate) fn after(mut self, child: impl Into<Pattern>) -> SpacingRuleBuilder<'a> {
        self.child = Some(child.into());
        self.loc = Some(SpaceLoc::After);
        self
    }
    /// The rule applies to the whitespace between the two nodes.
    pub(crate) fn between(
        mut self,
        left: impl Into<Pattern>,
        right: impl Into<Pattern>,
    ) -> SpacingRuleBuilder<'a> {
        self.between = Some((left.into(), right.into()));
        self.loc = Some(SpaceLoc::After);
        self
    }
    /// The rule applies if the `cond` is true.
    pub(crate) fn when(mut self, cond: fn(&SyntaxElement) -> bool) -> SpacingRuleBuilder<'a> {
        let pred = cond.into();
        let prev = self.child.take().unwrap();
        self.child = Some(prev & pred);
        self
    }
    /// Enforce single whitespace character.
    pub(crate) fn single_space(self) -> &'a mut SpacingDsl {
        self.finish(SpaceValue::Single)
    }
    pub(crate) fn single_space_or_optional_newline(self) -> &'a mut SpacingDsl {
        self.finish(SpaceValue::SingleOptionalNewline)
    }
    pub(crate) fn no_space_or_optional_newline(self) -> &'a mut SpacingDsl {
        self.finish(SpaceValue::NoneOptionalNewline)
    }
    /// Enforce the absence of any space.
    pub(crate) fn no_space(self) -> &'a mut SpacingDsl {
        self.finish(SpaceValue::None)
    }
    /// Enforce a single whitespace or newline character.
    pub(crate) fn single_space_or_newline(self) -> &'a mut SpacingDsl {
        self.finish(SpaceValue::SingleOrNewline)
    }
    /// Enforce an
    /// absence of whitespace or a newline character.
    pub(crate) fn no_space_or_newline(self) -> &'a mut SpacingDsl {
        self.finish(SpaceValue::NoneOrNewline)
    }
    fn finish(self, value: SpaceValue) -> &'a mut SpacingDsl {
        assert!(self.between.is_some() ^ self.child.is_some());
        if let Some((left, right)) = self.between {
            let child = {
                let left = left.clone();
                let right = right.clone();
                left & Pattern::from(move |it: &SyntaxElement| {
                    next_non_whitespace_sibling(it).map(|it| right.matches(&it)) == Some(true)
                })
            };
            let rule = SpacingRule {
                pattern: child.with_parent(self.parent.clone()),
                space: Space { value, loc: SpaceLoc::After },
            };
            self.dsl.rule(rule);

            let child = right
                & Pattern::from(move |it: &SyntaxElement| {
                    prev_non_whitespace_sibling(it).map(|it| left.matches(&it)) == Some(true)
                });
            let rule = SpacingRule {
                pattern: child.with_parent(self.parent),
                space: Space { value, loc: SpaceLoc::Before },
            };
            self.dsl.rule(rule);
        } else {
            let rule = SpacingRule {
                pattern: self.child.unwrap().with_parent(self.parent),
                space: Space { value, loc: self.loc.unwrap() },
            };
            self.dsl.rule(rule);
        }
        self.dsl
    }
}

pub(crate) fn prev_non_whitespace_sibling(element: &SyntaxElement) -> Option<SyntaxElement> {
    successors(element.prev_sibling_or_token(), |it| it.prev_sibling_or_token())
        .find(|it| it.kind() != WHITESPACE)
}

pub(crate) fn next_non_whitespace_sibling(element: &SyntaxElement) -> Option<SyntaxElement> {
    successors(element.next_sibling_or_token(), |it| it.next_sibling_or_token())
        .find(|it| it.kind() != WHITESPACE)
}
