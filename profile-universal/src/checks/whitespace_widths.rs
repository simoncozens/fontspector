use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use read_fonts::TableProvider;
use skrifa::MetadataProvider;

#[check(
    id = "whitespace_widths",
    rationale = "
        If the space and nbspace glyphs have different widths, then Google Workspace
        has problems with the font.

        The nbspace is used to replace the space character in multiple situations in
        documents; such as the space before punctuation in languages that do that. It
        avoids the punctuation to be separated from the last word and go to next line.

        This is automatic substitution by the text editors, not by fonts. It's also used
        by designers in text composition practice to create nicely shaped paragraphs.
        If the space and the nbspace are not the same width, it breaks the text
        composition of documents.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3843 https://github.com/fonttools/fontbakery/issues/4829",
    title = "Space and non-breaking space have the same width?"
)]
fn whitespace_widths(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    if let (Some(space), Some(nbspace)) = (
        f.font().charmap().map(0x0020u32),
        f.font().charmap().map(0x00A0u32),
    ) {
        let space_width = f.font().hmtx()?.advance(space).unwrap_or(0);
        let nbsp_width = f.font().hmtx()?.advance(nbspace).unwrap_or(0);
        Ok(if space_width == nbsp_width {
            Status::just_one_pass()
        } else {
            let space_name = f.glyph_name_for_id_synthesise(space);
            let nbsp_name = f.glyph_name_for_id_synthesise(nbspace);
            Status::just_one_fail("different-widths",
            &format!("The space glyph named {space_name} is {space_width} font units wide, non-breaking space named ({nbsp_name}) is {nbsp_width} font units wide, and both should be positive and the same. GlyphsApp has \"Sidebearing arithmetic\" (https://glyphsapp.com/tutorials/spacing) which allows you to set the non-breaking space width to always equal the space width.")
        )
        })
    } else {
        skip!("missing-glyphs", "Space and nbspace not found in font");
    }
}
