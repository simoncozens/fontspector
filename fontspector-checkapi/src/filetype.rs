use crate::{Testable, TestableCollection};
use glob_match::glob_match;

/// A file type that Fontrefinery can test.
///
/// This is a little bit strange, but the outcome we want is:
/// a) an instantiated unit struct that can be attached to a check, so
/// we can specify what file type that check applies to (we can't do it
/// with generics and trait cleverness, because we need to treat `Check`s
/// as homogenous types so we can put them into vecs etc.), and b)
/// the ability to turn a `Testable` into some other struct which is more
/// conducive to performing operations on that testable. (i.e. `TTF` can
/// turn a `Testable` into a `TestFont`.)
pub struct FileType<'a> {
    pub pattern: &'a str,
}
impl FileType<'_> {
    pub fn new(pattern: &str) -> FileType {
        FileType { pattern }
    }

    pub fn applies(&self, file: &Testable) -> bool {
        if let Some(basename) = file.basename() {
            glob_match(self.pattern, &basename)
        } else {
            false
        }
    }
}

pub trait FileTypeConvert<'a, T: 'a> {
    #[allow(clippy::wrong_self_convention)]
    fn from_testable(&self, t: &'a Testable) -> Option<T>;

    #[allow(clippy::wrong_self_convention)]
    fn from_collection(&self, t: &'a TestableCollection) -> Vec<T> {
        t.iter().filter_map(|f| self.from_testable(f)).collect()
    }
}
