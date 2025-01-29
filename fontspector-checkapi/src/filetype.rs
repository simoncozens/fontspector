use crate::{Testable, TestableCollection};
use glob_match::glob_match;

/// A file type that Fontrefinery can test.
///
/// This is a little bit strange, but the outcome we want is:
/// a) an instantiated unit struct that can be attached to a check, so
/// we can specify what file type that check applies to (we can't do it
/// with generics and trait cleverness, because we need to treat `Check`s
/// as homogenous types so we can put them into vecs etc; and we
/// can't do it with enums because we want the file types to be dynamically
/// defined), and b) the ability to turn a `Testable` into some other struct
/// which is more conducive to performing operations on that testable.
/// (i.e. `TTF` can turn a `Testable` into a `TestFont`.)
pub struct FileType<'a> {
    /// A glob pattern to match against the file name
    pub pattern: &'a str,
}
impl FileType<'_> {
    /// Create a new file type with a glob pattern
    pub fn new(pattern: &str) -> FileType {
        FileType { pattern }
    }

    /// Check if this file type applies to a testable
    pub fn applies(&self, file: &Testable) -> bool {
        if let Some(basename) = file.basename() {
            glob_match(self.pattern, &basename)
        } else {
            false
        }
    }
}

/// Convert a generic [Testable] into a specific file type
pub trait FileTypeConvert<'a, T: 'a> {
    #[allow(clippy::wrong_self_convention)]
    /// Convert a single [Testable] into a specific type
    fn from_testable(&self, t: &'a Testable) -> Option<T>;

    #[allow(clippy::wrong_self_convention)]
    /// Convert a collection of [Testable]s into a specific type
    fn from_collection(&self, t: &'a TestableCollection) -> Vec<T> {
        t.iter().filter_map(|f| self.from_testable(f)).collect()
    }
}
