use crate::{filetype::FileTypeConvert, FileType, Testable};
use read_fonts::{tables::os2::SelectionFlags, TableProvider};
use skrifa::{
    charmap::Charmap,
    font::FontRef,
    string::{LocalizedStrings, StringId},
    MetadataProvider, Tag,
};
use std::{
    collections::HashSet,
    error::Error,
    path::{Path, PathBuf},
};

pub struct TestFont<'a> {
    pub filename: PathBuf,
    font_data: &'a [u8],
    _codepoints: HashSet<u32>,
}

pub const TTF: FileType = FileType { pattern: "*.ttf" };

impl<'a> FileTypeConvert<'a, TestFont<'a>> for FileType<'a> {
    fn from_testable(&self, t: &'a Testable) -> Option<TestFont<'a>> {
        self.applies(t)
            .then(|| TestFont::new_from_data(&t.filename, &t.contents))
            .transpose()
            .unwrap_or(None)
    }
}

impl TestFont<'_> {
    pub fn new_from_data<'a>(
        filename: &Path,
        font_data: &'a [u8],
    ) -> Result<TestFont<'a>, Box<dyn Error>> {
        let font = FontRef::new(font_data)?;
        let _codepoints = Charmap::new(&font).mappings().map(|(u, _gid)| u).collect();
        Ok(TestFont {
            filename: filename.to_path_buf(),
            font_data,
            _codepoints,
        })
    }

    pub fn font(&self) -> FontRef {
        #[allow(clippy::expect_used)] // We just tested for it in the initializer
        FontRef::new(self.font_data).expect("Can't happen")
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

    pub fn codepoints(&self) -> &HashSet<u32> {
        &self._codepoints
    }
}
