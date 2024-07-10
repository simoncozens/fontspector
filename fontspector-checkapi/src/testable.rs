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
}
