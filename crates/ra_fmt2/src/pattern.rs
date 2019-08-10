use ra_syntax::{SyntaxElement, SyntaxKind};

use std::{
    collections::{HashMap, HashSet},
    fmt, iter, ops,
    sync::Arc,
};

#[derive(Clone)]
pub(crate) struct Pattern {
    kinds: Option<HashSet<SyntaxKind>>,
    pred: Arc<dyn (Fn(&SyntaxElement) -> bool)>,
}

impl fmt::Debug for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pattern {:?}", self.kinds)
    }
}

impl AsRef<Pattern> for Pattern {
    fn as_ref(&self) -> &Pattern {
        self
    }
}

impl Pattern {
    fn new(
        kinds: Option<HashSet<SyntaxKind>>,
        pred: impl Fn(&SyntaxElement) -> bool + 'static,
    ) -> Pattern {
        Pattern { kinds, pred: Arc::new(pred) }
    }

    fn filter_by_kind(kinds: impl Iterator<Item = SyntaxKind>) -> Pattern {
        Pattern::new(Some(kinds.collect()), |_| true)
    }

    /// Creates a pattern which matches the same elements as `self` with the
    /// additional constraint that their parent matches `parent`.
    pub(crate) fn with_parent(self, parent: Pattern) -> Pattern {
        let Pattern { kinds, pred } = self;
        Pattern::new(kinds, move |element| {
            (pred)(element)
                && element
                    .parent()
                    .map(|it| {
                        // println!("{:?}", it);
                        it
                    })
                    .map(|it| parent.matches(&it.into()))
                    == Some(true)
        })
    }

    /// Checks if this pattern matches an element
    pub(crate) fn matches(&self, element: &SyntaxElement) -> bool {
        if let Some(kinds) = self.kinds.as_ref() {
            if !kinds.contains(&element.kind()) {
                return false;
            }
        }
        (self.pred)(element)
    }
}

impl ops::BitAnd for Pattern {
    type Output = Self;
    fn bitand(self, other: Pattern) -> Pattern {
        let kinds = match (self.kinds, other.kinds) {
            (Some(lhs), Some(rhs)) => Some(lhs.intersection(&rhs).cloned().collect::<HashSet<_>>()),
            (Some(k), None) | (None, Some(k)) => Some(k),
            (None, None) => None,
        };
        let (p1, p2) = (self.pred, other.pred);
        Pattern::new(kinds, move |ele| p1(ele) && p2(ele))
    }
}

/// Construct pattern from closure.
impl<F> From<F> for Pattern
where
    F: for<'a> Fn(&SyntaxElement) -> bool + 'static,
{
    fn from(f: F) -> Pattern {
        Pattern::new(None, f)
    }
}

/// Construct pattern from a single `SyntaxKind`.
impl From<SyntaxKind> for Pattern {
    fn from(kind: SyntaxKind) -> Pattern {
        Pattern::filter_by_kind(iter::once(kind))
    }
}

/// Construct pattern from several `SyntaxKind`s (using slices).
impl From<&'_ [SyntaxKind]> for Pattern {
    fn from(kinds: &[SyntaxKind]) -> Pattern {
        Pattern::filter_by_kind(kinds.iter().cloned())
    }
}

/// Construct pattern from several `SyntaxKind`s (using arrays).
macro_rules! from_array {
    ($($arity:literal),*) => {$(
        impl From<[SyntaxKind; $arity]> for Pattern {
            fn from(kinds: [SyntaxKind; $arity]) -> Pattern {
                Pattern::from(&kinds[..])
            }
        }
    )*}
}
from_array!(0, 1, 2, 3, 4, 5, 6, 7, 8);

/// `PatternSet` allows to match many patterns at the same time efficiently.
///
/// This is generic over `P: AsRef<Pattern>`, so it works with any type which
/// contains a pattern.
pub(crate) struct PatternSet<P> {
    by_kind: HashMap<SyntaxKind, Vec<P>>,
    unconstrained: Vec<P>,
}

impl<'a, P: AsRef<Pattern>> PatternSet<&'a P> {
    pub(crate) fn new(patterns: impl Iterator<Item = &'a P>) -> PatternSet<&'a P> {
        let mut by_kind: HashMap<SyntaxKind, Vec<&'a P>> = HashMap::new();
        let mut unconstrained = vec![];
        patterns.for_each(|item| {
            let pat: &Pattern = item.as_ref();
            if let Some(kinds) = &pat.kinds {
                for &kind in kinds {
                    by_kind.entry(kind).or_default().push(item)
                }
            } else {
                unconstrained.push(item)
            }
        });
        PatternSet { by_kind, unconstrained }
    }

    /// Returns an iterator of patterns that match
    pub(crate) fn matching<'b>(
        &'b self,
        element: SyntaxElement,
    ) -> impl Iterator<Item = &'a P> + 'b {
        self.by_kind
            .get(&element.kind())
            .into_iter()
            .flat_map(|vec| vec.iter())
            .chain(self.unconstrained.iter())
            .map(|&p| p)
            .filter(move |p| p.as_ref().matches(&element))
    }
}
