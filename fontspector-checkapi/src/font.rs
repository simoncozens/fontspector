// use crate::constants::RIBBI_STYLE_NAMES;
use read_fonts::{tables::os2::SelectionFlags, TableProvider};
use skrifa::{
    font::FontRef,
    string::{LocalizedStrings, StringId},
    Tag,
    MetadataProvider,
};
use std::error::Error;

pub struct TestFont {
    pub filename: String,
    // pub font: FontRef<'a>,
    font_data: Vec<u8>,
}

impl TestFont {
    pub fn new(filename: &str) -> Result<Self, Box<dyn Error>> {
        let font_data = std::fs::read(filename).expect("Couldn't open file");
        Ok(Self {
            filename: filename.to_owned(),
            // font: FontRef::new(&font_data)?,
            font_data,
        })
    }

    pub fn font(&self) -> FontRef {
        FontRef::new(&self.font_data).expect("Can't parse font")
    }

    pub fn style(&self) -> Option<&str> {
        Some("Regular")
    }

    pub(crate) fn get_os2_fsselection(&self) -> Result<SelectionFlags, Box<dyn Error>> {
        let os2 = self.font().os2()?;
        Ok(os2.fs_selection())
    }

    pub fn get_name_entry_strings(&self, name_id: StringId) -> LocalizedStrings {
        self.font().localized_strings(name_id)
    }

    pub fn is_variable_font(&self) -> bool {
        self.font().table_data(Tag::new(b"fvar")).is_some()
    }
}

pub struct FontCollection<'a>(pub Vec<&'a TestFont>);

impl FontCollection<'_> {
    // pub fn ribbi_fonts(&self) -> FontCollection {
    //     let filtered: Vec<&TestFont> = self
    //         .0
    //         .iter()
    //         .copied()
    //         .filter(|x| RIBBI_STYLE_NAMES.contains(&x.style().unwrap_or("None")))
    //         .collect();
    //     FontCollection(filtered)
    // }
    pub fn iter(&self) -> std::slice::Iter<'_, &TestFont> {
        self.0.iter()
    }
}
