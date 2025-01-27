use fontspector_checkapi::{pens::XDeltaPen, prelude::*, skip, testfont, FileTypeConvert};
use skrifa::MetadataProvider;

#[check(
    id = "opentype/slant_direction",
    rationale = "
        The 'slnt' axis values are defined as negative values for a clockwise (right)
        lean, and positive values for counter-clockwise lean. This is counter-intuitive
        for many designers who are used to think of a positive slant as a lean to
        the right.

        This check ensures that the slant axis direction is consistent with the specs.

        https://docs.microsoft.com/en-us/typography/opentype/spec/dvaraxistag_slnt
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/3910",
    title = "Checking direction of slnt axis angles"
)]
fn slant_direction(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.is_variable_font(), "not-variable", "Not a variable font");
    let (_a, slnt_min, _dflt, slnt_max) = f
        .axis_ranges()
        .find(|(a, _min, _dflt, _max)| a == "slnt")
        .ok_or_else(|| CheckError::skip("no-slnt", "No 'slnt' axis found"))?;
    let h_id = f
        .font()
        .charmap()
        .map('H')
        .ok_or_else(|| CheckError::skip("no-H", "No H glyph in font"))?;
    // Get outline at slnt_max
    let mut max_pen = XDeltaPen::new();
    f.draw_glyph(h_id, &mut max_pen, vec![("slnt", slnt_max)])?;
    let mut min_pen = XDeltaPen::new();
    f.draw_glyph(h_id, &mut min_pen, vec![("slnt", slnt_min)])?;
    if min_pen.x_delta() > max_pen.x_delta() {
        Ok(Status::just_one_pass())
    } else {
        Ok(Status::just_one_fail(
            "positive-value-for-clockwise-lean",
            "The right-leaning glyphs have a positive 'slnt' axis value, which is likely a mistake. It needs to be negative to lean rightwards.",
        ))
    }
}
