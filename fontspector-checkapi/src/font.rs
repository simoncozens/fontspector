use crate::{filetype::FileTypeConvert, FileType, Testable};
use read_fonts::{tables::os2::SelectionFlags, TableProvider};
use skrifa::{
    charmap::Charmap,
    font::FontRef,
    string::{LocalizedStrings, StringId},
    GlyphId, MetadataProvider, Tag,
};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    path::{Path, PathBuf},
};

pub struct TestFont<'a> {
    pub filename: PathBuf,
    font_data: &'a [u8],
    // things it's worth caching
    _codepoints: HashSet<u32>,
    _instance_coordinates: Vec<(String, HashMap<String, f32>)>,
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
        let _axes = font.axes();
        let _instance_coordinates = font
            .named_instances()
            .iter()
            .map(|ni| {
                let instance_name = font
                    .localized_strings(ni.subfamily_name_id())
                    .english_or_first()
                    .map(|s| s.chars().collect::<String>())
                    .unwrap_or("Unnamed".to_string());
                let coords = ni
                    .user_coords()
                    .zip(font.axes().iter())
                    .map(|(coord, axis)| (axis.tag().to_string(), coord));
                (instance_name, coords.collect())
            })
            .collect();
        Ok(TestFont {
            filename: filename.to_path_buf(),
            font_data,
            _codepoints,
            _instance_coordinates,
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

    pub fn glyph_name_for_id(&self, gid: GlyphId) -> String {
        if let Ok(Some(name)) = self
            .font()
            .post()
            .map(|post| post.glyph_name(gid).map(|x| x.to_string()))
        {
            name
        } else {
            format!("gid{:}", gid)
        }
    }

    pub fn glyph_name_for_unicode(&self, u: impl Into<u32>) -> Option<String> {
        self.font()
            .charmap()
            .map(u)
            .map(|gid| self.glyph_name_for_id(gid))
    }

    pub fn is_variable_font(&self) -> bool {
        self.has_table(b"fvar")
    }

    pub fn codepoints(&self) -> &HashSet<u32> {
        &self._codepoints
    }

    /// Returns an iterator over the named instances in the font.
    ///
    /// Each item is a tuple of the instance name and a map of axis tag to user coordinate value.
    pub fn named_instances(&self) -> impl Iterator<Item = &(String, HashMap<String, f32>)> + '_ {
        self._instance_coordinates.iter()
    }
}
