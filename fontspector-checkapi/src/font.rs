use crate::{constants::RIBBI_STYLE_NAMES, filetype::FileTypeConvert, FileType, Testable};
use read_fonts::{tables::os2::SelectionFlags, TableProvider};
use skrifa::{
    font::FontRef,
    string::{LocalizedStrings, StringId},
    MetadataProvider, Tag,
};
use std::error::Error;

pub struct TestFont {
    font_data: Vec<u8>,
}

pub const TTF: FileType = FileType { pattern: "*.ttf" };

impl<'a> FileTypeConvert<TestFont> for FileType<'a> {
    fn from_testable(&self, t: &Testable) -> Option<TestFont> {
        if self.applies(t) {
            let font_data = std::fs::read(&t.filename).expect("Couldn't open file");
            Some(TestFont { font_data })
        } else {
            None
        }
    }
}

impl TestFont {
    pub fn font(&self) -> FontRef {
        FontRef::new(&self.font_data).expect("Can't parse font")
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
