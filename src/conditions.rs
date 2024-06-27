use crate::TestFont;

use skrifa::Tag;

pub fn is_variable_font(f: &TestFont) -> bool {
    f.font().table_data(Tag::new(b"fvar")).is_some()
}
