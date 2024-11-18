use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;

#[check(
    id = "opentype/loca/maxp_num_glyphs",
    title = "Does the number of glyphs in the loca table match the maxp table?",
    rationale = "
        The 'maxp' table contains various statistics about the font, including the
        number of glyphs in the font. The 'loca' table contains the offsets to the
        locations of the glyphs in the font. The number of offsets in the 'loca' table
        should match the number of glyphs in the 'maxp' table. A failure here indicates
        a problem with the font compiler.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",  // legacy check
)]
fn loca_maxp_num_glyphs(t: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(t);

    let loca = font
        .font()
        .loca(None)
        .map_err(|_| CheckError::skip("no-loca", "loca table not found"))?;
    if loca.len() != font.glyph_count {
        return Ok(Status::just_one_fail(
            "corrupt",
            "Corrupt \"loca\" table or wrong numGlyphs in \"maxp\" table.",
        ));
    } else {
        return Ok(Status::just_one_pass());
    }
}
