#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Testable {
    pub filename: String,
    pub source: Option<String>,
    #[cfg(target_family = "wasm")]
    pub contents: Vec<u8>,
}

impl Testable {
    #[cfg(not(target_family = "wasm"))]
    pub fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_owned(),
            source: None,
        }
    }

    #[cfg(not(target_family = "wasm"))]
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
