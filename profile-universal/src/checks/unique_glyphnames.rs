use std::collections::HashSet;

use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use read_fonts::{
    types::{Version16Dot16, CFF_SFNT_VERSION, TT_SFNT_VERSION},
    TableProvider,
};

#[check(
    id = "unique_glyphnames",
    rationale = "Duplicate glyph names prevent font installation on Mac OS X.",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Font contains unique glyph names?"
)]
fn unique_glyphnames(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(
        f.font().table_directory.sfnt_version() == TT_SFNT_VERSION
            && f.font().post()?.version() == Version16Dot16::new(3, 0),
        "tt-post3",
        "TrueType fonts with a format 3 post table contain no glyph names."
    );
    skip!(
        f.font().table_directory.sfnt_version() == CFF_SFNT_VERSION
            && f.has_table(b"CFF2")
            && f.font().post()?.version() == Version16Dot16::new(3, 0),
        "cff2-post3",
        "OpenType-CFF2 fonts with a format 3 post table contain no glyph names."
    );
    let mut duplicates = vec![];
    let mut seen_glyphs = HashSet::new();
    for glyph in f.all_glyphs() {
        if let Some(name) = f.glyph_name_for_id(glyph) {
            if seen_glyphs.contains(&name) {
                duplicates.push(name);
            } else {
                seen_glyphs.insert(name);
            }
        } else {
            // We are synthesising, stop
            break;
        }
    }
    if duplicates.is_empty() {
        Ok(Status::just_one_pass())
    } else {
        Ok(Status::just_one_fail(
            "duplicated-glyph-names",
            &format!(
                "
            These glyph names occur more than once: {}",
                bullet_list(context, duplicates)
            ),
        ))
    }
}
