use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};

#[check(
    id = "whitespace_glyphs",
    rationale = "
        The OpenType specification recommends that fonts should contain
        glyphs for the following whitespace characters:

        - U+0020 SPACE
        - U+00A0 NO-BREAK SPACE

        The space character is required for text processing, and the no-break
        space is useful to prevent line breaks at its position. It is also
        recommended to have a glyph for the tab character (U+0009) and the
        soft hyphen (U+00AD), but these are not mandatory.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Font contains glyphs for whitespace characters?"
)]
fn whitespace_glyphs(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    for c in [0x20, 0x0A0] {
        if !f.codepoints().contains(&c) {
            problems.push(Status::fail(
                &format!("missing-whitespace-glyph-0x{:04X}", c),
                &format!("Whitespace glyph missing for codepoint 0x{:04X}", c),
            ))
        }
    }
    return_result(problems)
}
