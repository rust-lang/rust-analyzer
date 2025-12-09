//! Classifies loaded files by their longest configured path prefix.

use fst::{IntoStreamer, Streamer};

use crate::VfsPath;

/// Classifies paths by their longest configured prefix.
///
/// # Example
/// ```rust
/// # use vfs::{VfsPath, path_classifier::PathClassifierBuilder};
/// let mut builder = PathClassifierBuilder::default();
/// builder.add_root(vec![VfsPath::new_virtual_path("/src".to_string())]);
/// let config = builder.build();
/// assert_eq!(config.classify(&VfsPath::new_virtual_path("/src/main.rs".to_string())), 0);
/// assert_eq!(config.classify(&VfsPath::new_virtual_path("/build.rs".to_string())), 1);
/// ```
#[derive(Debug)]
pub struct PathClassifier {
    /// Includes the fallback root for paths that match no configured prefix.
    root_count: usize,
    /// Maps encoded path prefixes to root indices.
    map: fst::Map<Vec<u8>>,
}

impl Default for PathClassifier {
    fn default() -> Self {
        PathClassifier::builder().build()
    }
}

impl PathClassifier {
    /// Returns a builder for `PathClassifier`.
    pub fn builder() -> PathClassifierBuilder {
        PathClassifierBuilder::default()
    }

    /// Returns the root index for the given `path`.
    pub fn classify(&self, path: &VfsPath) -> usize {
        self.classify_with_scratch(path, &mut Vec::new())
    }

    /// Returns the root index for the given `path`, or `None` if it matches no
    /// configured prefix and so falls into the catch-all root.
    pub fn classify_configured(&self, path: &VfsPath) -> Option<usize> {
        let idx = self.classify(path);
        (idx != self.len() - 1).then_some(idx)
    }

    /// Returns the number of configured roots, including the fallback root.
    fn len(&self) -> usize {
        self.root_count
    }

    /// Get the lexicographically ordered vector of the underlying map.
    pub fn roots(&self) -> Vec<(Vec<u8>, u64)> {
        self.map.stream().into_byte_vec()
    }

    /// Returns the root index for the given `path`.
    ///
    /// `scratch_space` is used as a buffer and will be entirely replaced.
    fn classify_with_scratch(&self, path: &VfsPath, scratch_space: &mut Vec<u8>) -> usize {
        // `path` is a file, but r-a only cares about the containing directory. We don't
        // want `/foo/bar_baz.rs` to be attributed to file-root directory `/foo/bar`.
        let path = path.parent().unwrap_or_else(|| path.clone());

        scratch_space.clear();
        path.encode(scratch_space);
        let automaton = PrefixOf::new(scratch_space.as_slice());
        let mut longest_prefix = self.len() - 1;
        let mut stream = self.map.search(automaton).into_stream();
        while let Some((_, v)) = stream.next() {
            longest_prefix = v as usize;
        }
        longest_prefix
    }
}

/// Builder for [`PathClassifier`].
#[derive(Default)]
pub struct PathClassifierBuilder {
    roots: Vec<Vec<VfsPath>>,
}

impl PathClassifierBuilder {
    /// Returns the number of configured roots.
    pub fn len(&self) -> usize {
        self.roots.len()
    }

    /// Adds the path prefixes that identify one root.
    pub fn add_root(&mut self, roots: Vec<VfsPath>) {
        self.roots.push(roots);
    }

    /// Build the `PathClassifier`.
    pub fn build(self) -> PathClassifier {
        let root_count = self.roots.len() + 1;
        let map = {
            let mut entries = Vec::new();
            for (i, paths) in self.roots.into_iter().enumerate() {
                for p in paths {
                    let mut buf = Vec::new();
                    p.encode(&mut buf);
                    entries.push((buf, i as u64));
                }
            }
            entries.sort();
            entries.dedup_by(|(a, _), (b, _)| a == b);
            fst::Map::from_iter(entries).unwrap()
        };
        PathClassifier { root_count, map }
    }
}

/// Implements [`fst::Automaton`]
///
/// It will match if `prefix_of` is a prefix of the given data.
struct PrefixOf<'a> {
    prefix_of: &'a [u8],
}

impl<'a> PrefixOf<'a> {
    /// Creates a new `PrefixOf` from the given slice.
    fn new(prefix_of: &'a [u8]) -> Self {
        Self { prefix_of }
    }
}

impl fst::Automaton for PrefixOf<'_> {
    type State = usize;
    fn start(&self) -> usize {
        0
    }
    fn is_match(&self, &state: &usize) -> bool {
        state != !0
    }
    fn can_match(&self, &state: &usize) -> bool {
        state != !0
    }
    fn accept(&self, &state: &usize, byte: u8) -> usize {
        if self.prefix_of.get(state) == Some(&byte) { state + 1 } else { !0 }
    }
}

#[cfg(test)]
mod tests;
