use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;

#[check(
    id = "linegaps",
    rationale = "
        The LineGap value is a space added to the line height created by the union
        of the (typo/hhea)Ascender and (typo/hhea)Descender. It is handled differently
        according to the environment.

        This leading value will be added above the text line in most desktop apps.
        It will be shared above and under in web browsers, and ignored in Windows
        if Use_Typo_Metrics is disabled.

        For better linespacing consistency across platforms,
        (typo/hhea)LineGap values must be 0.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4133 and https://googlefonts.github.io/gf-guide/metrics.html",
    title = "Checking Vertical Metric linegaps."
)]
fn linegaps(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);

    let os2 = f
        .font()
        .os2()
        .map_err(|_| CheckError::Error("OS/2 table missing".to_string()))?;
    let hhea = f
        .font()
        .hhea()
        .map_err(|_| CheckError::Error("hhea table missing".to_string()))?;
    let mut problems = vec![];
    if hhea.line_gap().to_i16() != 0 {
        problems.push(Status::warn("hhea", "hhea lineGap is not equal to 0."));
    }
    if os2.s_typo_line_gap() != 0 {
        problems.push(Status::warn("OS/2", "OS/2 sTypoLineGap is not equal to 0."));
    }
    return_result(problems)
}
