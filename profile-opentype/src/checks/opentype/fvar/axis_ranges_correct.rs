use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use skrifa::MetadataProvider;

#[check(
    id = "opentype/fvar/axis_ranges_correct",
    title = "Axes and named instances fall within correct ranges?",
    rationale = "According to the Open-Type spec's registered design-variation tags, instances in a variable font should have certain prescribed values.
        If a variable font has a 'wght' (Weight) axis, the valid coordinate range is 1-1000.
        If a variable font has a 'wdth' (Width) axis, the valid numeric range is strictly greater than zero.
        If a variable font has a 'slnt' (Slant) axis, then the coordinate of its 'Regular' instance is required to be 0.
        If a variable font has a 'ital' (Slant) axis, then the coordinate of its 'Regular' instance is required to be 0.",
    proposal = "https://github.com/fonttools/fontbakery/issues/2572"
)]
fn axis_ranges_correct(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.is_variable_font(), "not-variable", "Not a variable font");

    let mut problems = vec![];
    for (name, location) in f.named_instances() {
        if let Some(wght) = location.get("wght") {
            if !(1.0..=1000.0).contains(wght) {
                problems.push(Status::fail(
                    "wght-out-of-range",
                    &format!(
                        "Instance {} has wght coordinate of {}, expected between 1 and 1000",
                        name, wght
                    ),
                ));
            }
        }
        if let Some(wdth) = location.get("wdth") {
            if *wdth < 1.0 {
                problems.push(Status::fail(
                    "wdth-out-of-range",
                    &format!(
                        "Instance {} has wdth coordinate of {}, expected at least 1",
                        name, wdth
                    ),
                ));
            }
            if *wdth > 1000.0 {
                problems.push(Status::warn(
                    "wdth-greater-than-1000",
                    &format!(
                        "Instance {} has wdth coordinate of {}, which is valid but unusual",
                        name, wdth
                    ),
                ));
            }
        }
    }

    let axes = f.font().axes();
    if let Some(ital) = axes.iter().find(|axis| axis.tag() == "ital") {
        if !(ital.min_value() == 0.0 && ital.max_value() == 1.0) {
            problems.push(Status::fail(
                "invalid-ital-range",
                &format!(
                    "The range of values for the \"ital\" axis in this font is {} to {}.
                    The italic axis range must be 0 to 1, where Roman is 0 and Italic 1.
                    If you prefer a bigger variation range consider using the \"Slant\" axis instead of \"Italic\".",
                    ital.min_value(), ital.max_value()
                ),
            ));
        }
    }

    if let Some(slnt) = axes.iter().find(|axis| axis.tag() == "slnt") {
        if !(slnt.min_value() < 0.0 && slnt.max_value() >= 0.0) {
            problems.push(Status::warn(
                "unusual-slnt-range",
                &format!(
                    "The range of values for the \"slnt\" axis in this font only allows positive coordinates (from {} to {}),
                    indicating that this may be a back slanted design, which is rare. If that's not the case, then
                    the \"slnt\" axis should be a range of negative values instead.",
                    slnt.min_value(), slnt.max_value()
                ),
            ));
        }
    }
    return_result(problems)
}
