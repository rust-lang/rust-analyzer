use std::iter;
use std::fmt;
use std::ops::Range;

/// A generalized `&'a str`, which is not necessary backed up by a contiguous
/// sequence of bytes.
pub trait Str<'a>:
    Clone + fmt::Display + fmt::Debug
    // doesn't work for str :(
    // + PartialEq<str>
    + for<'b> PartialEq<&'b str>
    // This should probably be Ord and Hash as well
{

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Slicing (aka random access by byte positions) seems to be a fundamental
    /// operation.
    ///
    /// For maximum performance, this probably should be
    /// type Slice: Str<'a> = Self;
    /// fn slice(&self, range: Range<usize>) -> Self::Slice;
    fn slice(&self, range: Range<usize>) -> Self;


    /// The main accessor to the str data
    type Chunks: Iterator<Item = &'a str>;
    fn chunks(&self) -> Self::Chunks;

    /// A bunch of helper methods mirroring those of `str`
    /// This should be generalized to `Pattern` API
    fn find(&self, c: char) -> Option<usize> {
        match self.chunks().try_fold(0, |offset, chunk| {
            if let Some(idx) = chunk.find(c) {
                return Err(offset + idx);
            }
            Ok(offset + chunk.len())
        }) {
            Err(idx) => Some(idx),
            Ok(len) => {
                debug_assert_eq!(len, self.len());
                None
            }
        }
    }

    fn contains(&self, c: char) -> bool {
        self.chunks().any(|chunk| chunk.contains(c))
    }

    /// Specialization helpers: all `Str` should be Display, Debug, Eq and Hash.
    /// We can easily write a blanket impl, but than we won't be able to write
    /// an `Str` impl for `str`. Maybe there's a nicer way to achieve this?
    fn display(this: &Self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        this.chunks()
            .try_for_each(|chunk| fmt::Display::fmt(chunk, f))
    }

    fn debug(this: &Self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\"")?;
        this.chunks()
            .try_for_each(|chunk| fmt::Display::fmt(&chunk.escape_debug(), f))?;
        f.write_str("\"")
    }

    fn eq(this: &Self, mut s: &str) -> bool {
        for chunk in this.chunks() {
            if !s.starts_with(chunk) {
                return false;
            }
            s = &s[chunk.len()..];
        }
        s.is_empty()
    }

}

/// A trivial implementation for `str`
impl<'a> Str<'a> for &'a str {
    fn len(&self) -> usize {
        <str>::len(*self)
    }
    fn slice(&self, range: Range<usize>) -> Self {
        &self[range]
    }
    type Chunks = iter::Once<&'a str>;
    fn chunks(&self) -> Self::Chunks {
        iter::once(*self)
    }
}

/// A somewhat silly, almost-trivial implementation for slices.
///
/// Unlike Java
/// (https://github.com/JetBrains/intellij-community/blob/e693fb087902be0df9866d392a5807de15c25a64/platform/util/src/com/intellij/util/text/CharSequenceSubSequence.java),
/// slicing needs to be a fundamental operation
#[derive(Clone)]
struct Substring<S> {
    base: S,
    range: Range<usize>,
}

impl<'a, S: Str<'a>> Str<'a> for Substring<S> {
    fn len(&self) -> usize {
        let Range { start, end } = self.range.clone();
        end - start
    }

    fn slice(&self, range: Range<usize>) -> Self {
        let start = self.range.start + range.start;
        let end = self.range.start + range.end;
        assert!(end <= self.range.end);
        Substring { base: self.base.clone(), range: start..end }
    }

    type Chunks = S::Chunks;
    fn chunks(&self) -> Self::Chunks {
        self.base.slice(self.range.clone()).chunks()
    }
}

impl<'a, S: Str<'a>> fmt::Display for Substring<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Str::display(self, f)
    }
}

impl<'a, S: Str<'a>> fmt::Debug for Substring<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Str::debug(self, f)
    }
}

impl<'a, S: Str<'a>> PartialEq<&str> for Substring<S> {
    fn eq(&self, s: &&str) -> bool {
        Str::eq(self, s)
    }
}
