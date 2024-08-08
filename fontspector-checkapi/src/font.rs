use crate::{constants::RIBBI_STYLE_NAMES, filetype::FileTypeConvert, FileType, Testable};
use read_fonts::{tables::os2::SelectionFlags, TableProvider};
use skrifa::{
    charmap::Charmap,
    font::FontRef,
    string::{LocalizedStrings, StringId},
    MetadataProvider, Tag,
};
use std::{collections::HashSet, error::Error, io::ErrorKind, path::Path};

pub struct TestFont {
    filename: String,
    font_data: Vec<u8>,
    _codepoints: HashSet<u32>,
    _sibling_filenames: Vec<String>,
}

pub const TTF: FileType = FileType { pattern: "*.ttf" };

impl<'a> FileTypeConvert<TestFont> for FileType<'a> {
    fn from_testable(&self, t: &Testable) -> Option<TestFont> {
        self.applies(t)
            .then(|| TestFont::new(&t.filename))
            .transpose()
            .unwrap_or(None)
    }
}

impl TestFont {
    pub fn new(filename: &str) -> std::io::Result<TestFont> {
        let font_data = std::fs::read(filename)?;
        let mut fnt = TestFont {
            filename: filename.to_string(),
            font_data,
            _codepoints: HashSet::default(),
            _sibling_filenames: vec![],
        };
        // Cache some stuff

        fnt._codepoints = Charmap::new(&fnt.font())
            .mappings()
            .map(|(u, _gid)| u)
            .collect();
        fnt._sibling_filenames = {
            // All other TTF files in same directory
            let directory = Path::new(&fnt.filename)
                .parent()
                .ok_or(std::io::Error::new(
                    ErrorKind::NotFound,
                    "parent directory not found",
                ))?;

            let paths = std::fs::read_dir(directory)?;
            paths
                .flatten()
                .filter(|x| x.path().extension().map_or(false, |ext| ext == "ttf"))
                .filter(|x| x.path().to_string_lossy() != fnt.filename)
                .map(|x| x.path().to_string_lossy().to_string())
                .collect()
        };
        if FontRef::new(&fnt.font_data).is_err() {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "Can't parse font",
            ));
        }
        Ok(fnt)
    }
    pub fn font(&self) -> FontRef {
        #[allow(clippy::expect_used)] // We just tested for it in the initializer
        FontRef::new(&self.font_data).expect("Can't happen")
    }

    pub fn style(&self) -> Option<&str> {
        Some("Regular")
    }

    pub fn has_table(&self, table: &[u8; 4]) -> bool {
        self.font().table_data(Tag::new(table)).is_some()
    }

    pub fn get_os2_fsselection(&self) -> Result<SelectionFlags, Box<dyn Error>> {
        let os2 = self.font().os2()?;
        Ok(os2.fs_selection())
    }

    pub fn get_name_entry_strings(&self, name_id: StringId) -> LocalizedStrings {
        self.font().localized_strings(name_id)
    }

    pub fn is_variable_font(&self) -> bool {
        self.has_table(b"fvar")
    }

    pub fn siblings(&self) -> Vec<TestFont> {
        self._sibling_filenames
            .iter()
            .flat_map(|x| TestFont::new(x))
            .collect()
    }

    pub fn codepoints(&self) -> &HashSet<u32> {
        &self._codepoints
    }
}

pub struct FontCollection<'a>(pub Vec<&'a TestFont>);

impl FontCollection<'_> {
    pub fn ribbi_fonts(&self) -> FontCollection {
        let filtered: Vec<&TestFont> = self
            .0
            .iter()
            .copied()
            .filter(|x| RIBBI_STYLE_NAMES.contains(&x.style().unwrap_or("None")))
            .collect();
        FontCollection(filtered)
    }
    pub fn iter(&self) -> std::slice::Iter<'_, &TestFont> {
        self.0.iter()
    }
}
