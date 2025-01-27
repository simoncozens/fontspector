use std::collections::HashSet;

use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::types::NameId;

#[check(
    id = "opentype/name/postscript_name_consistency",
    rationale = "
        The PostScript name entries in the font's 'name' table should be
        consistent across platforms.

        This is the TTF/CFF2 equivalent of the CFF 'name/postscript_vs_cff' check.
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/2394",
    title = "Name table ID 6 (PostScript name) must be consistent across platforms."
)]
fn postscript_name_consistency(t: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(t);
    // Fontbakery had this just for non-CFF fonts, but I think we don't want
    // inconsistent PostScript names in *any* font!
    let psnames: HashSet<_> = font
        .get_name_entry_strings(NameId::POSTSCRIPT_NAME)
        .collect();
    if psnames.len() > 1 {
        return Ok(Status::just_one_fail(
            "inconsistency",
            &format!(
                "More than one entry found for PostScript name; we got: {:?}",
                psnames.into_iter().collect::<Vec<String>>().join(", ")
            ),
        ));
    }
    Ok(Status::just_one_pass())
}
