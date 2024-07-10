use crate::Testable;
use glob_match::glob_match;
pub struct FileType<'a> {
    pub pattern: &'a str,
}
impl FileType<'_> {
    pub fn new(pattern: &str) -> FileType {
        FileType { pattern }
    }

    pub fn applies(&self, file: &Testable) -> bool {
        glob_match(self.pattern, &file.filename)
    }
}

pub trait FileTypeConvert<T> {
    #[allow(clippy::wrong_self_convention)]
    fn from_testable(&self, t: &Testable) -> Option<T>;
}
