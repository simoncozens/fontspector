use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use read_fonts::TableProvider;

#[check(
    id = "os2_metrics_match_hhea",
    rationale = "
        OS/2 and hhea vertical metric values should match. This will produce the
        same linespacing on Mac, GNU+Linux and Windows.

        - Mac OS X uses the hhea values.
        - Windows uses OS/2 or Win, depending on the OS or fsSelection bit value.

        When OS/2 and hhea vertical metrics match, the same linespacing results on
        macOS, GNU+Linux and Windows. Note that fixing this issue in a previously
        released font may cause reflow in user documents and unhappy users.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Checking OS/2 Metrics match hhea Metrics."
)]
fn os2_metrics_match_hhea(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);

    skip!(
        f.is_cjk_font(),
        "cjk-font",
        "Actually I'm not sure why we don't check this on CJK fonts."
    );

    let os2 = f
        .font()
        .os2()
        .map_err(|_| CheckError::Error("OS/2 table missing".to_string()))?;
    let hhea = f
        .font()
        .hhea()
        .map_err(|_| CheckError::Error("hhea table missing".to_string()))?;
    let mut problems = vec![];
    if os2.s_typo_ascender() != hhea.ascender().to_i16() {
        problems.push(Status::fail(
            "ascender",
            &format!(
                "OS/2 sTypoAscender ({}) and hhea ascent ({}) must be equal.",
                os2.s_typo_ascender(),
                hhea.ascender().to_i16()
            ),
        ));
    }
    if os2.s_typo_descender() != hhea.descender().to_i16() {
        problems.push(Status::fail(
            "descender",
            &format!(
                "OS/2 sTypoDescender ({}) and hhea descent ({}) must be equal.",
                os2.s_typo_descender(),
                hhea.descender().to_i16()
            ),
        ));
    }
    if os2.s_typo_line_gap() != hhea.line_gap().to_i16() {
        problems.push(Status::fail(
            "lineGap",
            &format!(
                "OS/2 sTypoLineGap ({}) and hhea lineGap ({}) must be equal.",
                os2.s_typo_line_gap(),
                hhea.line_gap().to_i16()
            ),
        ));
    }
    return_result(problems)
}
