#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Testable {
    pub filename: String,
    pub source: Option<String>,
}

impl Testable {
    pub fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_owned(),
            source: None,
        }
    }

    pub fn new_with_source(filename: &str, source: &str) -> Self {
        Self {
            filename: filename.to_owned(),
            source: Some(source.to_owned()),
        }
    }

    pub fn basename(&self) -> Option<String> {
        std::path::Path::new(&self.filename)
            .file_name()
            .and_then(|x| x.to_str())
            .map(|x| x.to_string())
    }
}
