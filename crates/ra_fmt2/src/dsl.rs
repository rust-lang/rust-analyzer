use crate::pattern::{Pattern, PatternSet};
use crate::trav_util::{next_non_whitespace_sibling, prev_non_whitespace_sibling};
use ra_syntax::{SyntaxElement, SyntaxKind::*};

/// `SpacingRule` describes whitespace requirements between `SyntaxElement`.
/// Note that it doesn't handle indentation (first whitespace on a line), there's
/// `IndentRule` for that.
#[derive(Debug, Clone)]
pub(crate) struct SpacingRule {
    /// An element to which this spacing rule applies.
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

/// A builder to conveniently specify a set of `SpacingRule`s.
#[derive(Debug, Default)]
pub(crate) struct SpacingDsl {
    pub(crate) rules: Vec<SpacingRule>,
    /// Vec of tuples of before and after examples of formatting rules.
    /// Used only when `cargo test` is run.
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
    /// Adds test cases only if `cargo test` is run.
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

///
/// 
/// 

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Modality {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IndentValue {
    Indent,
    IndentFromParent,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuleName(&'static str);

impl RuleName {
    fn new(name: &'static str) -> RuleName {
        assert!(name.chars().next().unwrap().is_uppercase(), "rule names start with capital letter");
        assert!(!name.ends_with('.'), "rule names should not end with '.'");
        RuleName(name)
    }
}

/// `IndentRule` describes how an element should be indented.
///
/// `IndentRule`s are only effective for elements which begin the line.
///
/// Note that currently we support only two kinds of indentation:
/// * the same, as parent (default)
/// * indent relative to the parent.
///
/// For this reason, `indent_value` is mostly unused.
#[derive(Debug)]
pub(crate) struct IndentRule {
    pub(crate) name: RuleName,
    pub(crate) parent: Pattern,

    /// Depending on `child_modality`, this pattern selects/discards elements
    pub(crate) child: Option<Pattern>,
    pub(crate) child_modality: Modality,
    /// Pattern that should match the anchoring element, relative to which we
    /// calculate the indent. Starts the indent level count from this node/token?? TODO
    pub(crate) anchor_pattern: Option<Pattern>,
    pub(crate) indent_value: IndentValue,
}

impl IndentRule {
    pub(super) fn matches(&self, element: &SyntaxElement) -> bool {
        let parent = match element.parent() {
            None => return false,
            Some(it) => it,
        };
        if !self.parent.matches(&parent.into()) {
            return false;
        }
        if let Some(child) = &self.child {
            child.matches(element) == (self.child_modality == Modality::Positive)
        } else {
            true
        }
    }
}

/// A builder to conveniently specify a set of `IndentRule`s.
#[derive(Default, Debug)]
pub(crate) struct IndentDsl {
    pub(crate) rules: Vec<IndentRule>,
    pub(crate) anchors: Vec<Pattern>,
    #[cfg(test)]
    pub(crate) tests: Vec<(&'static str, &'static str)>,
}

impl IndentDsl {
    /// Specifies that an element should be treated as indent anchor even if it
    /// isn't the first on the line.
    ///
    /// For example, in
    ///
    /// ```nix
    /// { foo ? bar
    /// , baz ? quux {
    ///     y = z;
    ///   }
    /// }
    /// ```
    ///
    /// we want to indent `y = z;` relative to `baz ? ...`, although it doesn't
    /// start on the first line.
    pub(crate) fn anchor(&mut self, pattern: impl Into<Pattern>) -> &mut IndentDsl {
        self.anchors.push(pattern.into());
        self
    }
    /// Adds a new indent rule with the given name
    pub(crate) fn rule<'a>(&'a mut self, rule_name: &'static str) -> IndentRuleBuilder<'a> {
        IndentRuleBuilder::new(self, rule_name)
    }
    pub(crate) fn test(&mut self, before: &'static str, after: &'static str) -> &mut IndentDsl {
        #[cfg(test)]
        {
            self.tests.push((before, after));
        }
        let _ = (before, after);
        self
    }
}

/// A builder to conveniently specify a single `IndentRule`.
pub(crate) struct IndentRuleBuilder<'a> {
    dsl: &'a mut IndentDsl,
    rule_name: &'static str,
    parent: Option<Pattern>,
    child: Option<Pattern>,
    child_modality: Modality,
    anchor_pattern: Option<Pattern>,
}

impl<'a> IndentRuleBuilder<'a> {
    fn new(dsl: &'a mut IndentDsl, rule_name: &'static str) -> IndentRuleBuilder<'a> {
        IndentRuleBuilder {
            dsl,
            rule_name,
            parent: None,
            child: None,
            child_modality: Modality::Positive,
            anchor_pattern: None,
        }
    }

    /// Rule applies if element's parent matches.
    pub(crate) fn inside(mut self, parent: impl Into<Pattern>) -> Self {
        let prev = self.parent.replace(parent.into());
        assert!(prev.is_none());
        self
    }

    /// Rule applies if element itself matches.
    pub(crate) fn matching(self, child: impl Into<Pattern>) -> Self {
        self.matching_modality(child.into(), Modality::Positive)
    }

    /// Rule applies if element itself does *not* match.
    pub(crate) fn not_matching(self, child: impl Into<Pattern>) -> Self {
        self.matching_modality(child.into(), Modality::Negative)
    }

    fn matching_modality(mut self, child: Pattern, child_modality: Modality) -> Self {
        let prev = self.child.replace(child);
        assert!(prev.is_none());
        self.child_modality = child_modality;
        self
    }

    /// Sets which indent the rule apply to.
    pub(crate) fn set(self, indent_value: IndentValue) -> &'a mut IndentDsl {
        let dsl = self.dsl;
        let name = self.rule_name;
        let rule = IndentRule {
            name: RuleName::new(name),
            parent: self.parent.unwrap_or_else(|| panic!("incomplete rule: {}", name)),
            child: self.child,
            child_modality: self.child_modality,
            anchor_pattern: self.anchor_pattern,
            indent_value,
        };
        dsl.rules.push(rule);
        dsl
    }

    /// Only apply this rule when `cond` is true for the anchor node, relative
    /// to which we compute indentation level.
    pub(crate) fn when_anchor(mut self, cond: fn(&SyntaxElement) -> bool) -> Self {
        self.anchor_pattern = Some(cond.into());
        self
    }
}
