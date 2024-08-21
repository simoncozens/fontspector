use std::path::{Path, PathBuf};

/// A single file to be tested
///
/// At this stage we do not care about the file type; this is sorted out later.
/// Testables should be provided to fontspector wrapped in a TestableCollection object,
/// for which see below.
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Testable {
    /// The filename of the binary.
    pub filename: PathBuf,
    /// The filename of the source which generated this binary.
    pub source: Option<PathBuf>,
    /// The binary contents.
    pub contents: Vec<u8>,
}

impl Testable {
    /// Create a new Testable from a filename.
    ///
    /// The contents are resolved from the filesystem.
    pub fn new<P: Into<PathBuf> + AsRef<Path>>(filename: P) -> Result<Self, std::io::Error> {
        let contents = std::fs::read(&filename)?;
        Ok(Self {
            filename: filename.into(),
            source: None,
            contents,
        })
    }

    /// Create a new Testable with a known source.
    pub fn new_with_source<P: Into<PathBuf> + AsRef<Path>>(
        filename: P,
        source: P,
    ) -> Result<Self, std::io::Error> {
        let contents = std::fs::read(&filename)?;
        Ok(Self {
            filename: filename.into(),
            source: Some(source.into()),
            contents: contents,
        })
    }

    /// Create a new Testable with known contents.
    ///
    /// This is used in the WASM version of fontspector.
    pub fn new_with_contents<P: Into<PathBuf> + AsRef<Path>>(
        filename: P,
        contents: Vec<u8>,
    ) -> Self {
        Self {
            filename: filename.into(),
            source: None,
            contents,
        }
    }

    /// Get the basename of the file.
    pub fn basename(&self) -> Option<String> {
        self.filename
            .file_name()
            .and_then(|x| x.to_str())
            .map(|x| x.to_string())
    }

    /// Get the extension of the file
    pub fn extension(&self) -> Option<String> {
        self.filename
            .extension()
            .and_then(|x| x.to_str())
            .map(|x| x.to_string())
    }
}

/// A related set of files which will be checked together.
///
/// For example: all the .TTF files in a family, together with a METADATA.pb and some HTML files.
/// Imagine it as a slice of a filesystem. This is the basic unit of testing.
#[derive(Debug, PartialEq, Eq)]
pub struct TestableCollection {
    pub testables: Vec<Testable>,
}

impl TestableCollection {
    pub fn from_filenames<P: Into<PathBuf> + AsRef<Path> + Clone>(
        filenames: &[P],
    ) -> Result<Self, std::io::Error> {
        let collection: Result<Vec<Testable>, _> = filenames
            .into_iter()
            .map(|x| Testable::new(x.clone()))
            .collect();
        Ok(Self {
            testables: collection?,
        })
    }

    pub fn from_testables(testables: Vec<Testable>) -> Self {
        Self { testables }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Testable> {
        self.testables.iter()
    }

    pub fn collection_and_files(&self) -> impl Iterator<Item = TestableType> {
        vec![TestableType::Collection(self)]
            .into_iter()
            .chain(self.testables.iter().map(|x| TestableType::Single(x)))
    }
}

pub enum TestableType<'a> {
    Single(&'a Testable),
    Collection(&'a TestableCollection),
}

impl TestableType<'_> {
    pub fn is_single(&self) -> bool {
        matches!(self, TestableType::Single(_))
    }
}
