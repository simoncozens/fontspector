use fontspector_checkapi::{
    pens::AreaPen, prelude::*, testfont, FileTypeConvert, DEFAULT_LOCATION,
};
use read_fonts::TableProvider;
use skrifa::GlyphId;

#[check(
    id = "empty_glyph_on_gid1_for_colrv0",
    rationale = "
        A rendering bug in Windows 10 paints whichever glyph is on GID 1 on top of
        some glyphs, colored or not. This only occurs for COLR version 0 fonts.

        Having a glyph with no contours on GID 1 is a practical workaround for that.

        See https://github.com/googlefonts/gftools/issues/609
    ",
    proposal = "https://github.com/googlefonts/gftools/issues/609 and https://github.com/fonttools/fontbakery/pull/3905",
    title = "Put an empty glyph on GID 1 right after the .notdef glyph for COLRv0 fonts."
)]
fn empty_glyph_on_gid1_for_colrv0(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut pen = AreaPen::new();
    f.draw_glyph(GlyphId::new(1), &mut pen, DEFAULT_LOCATION)?;
    if pen.area() != 0.0 && f.has_table(b"COLR") && f.font().colr()?.version() == 0 {
        Ok(Status::just_one_fail(
            "gid1-has-contours",
            "This is a COLR font. As a workaround for a rendering bug in Windows 10, it needs an empty glyph to be in GID 1. To fix this, please reorder the glyphs so that a glyph with no contours is on GID 1 right after the `.notdef` glyph. This could be the space glyph."
        ))
    } else {
        Ok(Status::just_one_pass())
    }
}
