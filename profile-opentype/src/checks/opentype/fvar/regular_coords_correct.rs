use std::collections::HashMap;

use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert, TestFont};

const REGULAR_COORDINATE_EXPECTATIONS: [(&str, f32); 4] = [
    ("wght", 400.0),
    ("wdth", 100.0),
    ("slnt", 0.0),
    ("ital", 0.0),
];

fn find_regular(f: TestFont) -> Option<HashMap<String, f32>> {
    let mut instance = f.named_instances().find(|(name, _loc)| name == "Regular");
    if instance.is_none() {
        instance = f.named_instances().find(|(name, _loc)| name == "Italic");
    }
    if instance.is_none() {
        // Should not happen but anyway
        instance = f
            .named_instances()
            .find(|(name, _loc)| name == "Regular Italic");
    }

    instance.map(|(_name, loc)| loc)
}

#[check(
    id = "opentype/fvar/regular_coords_correct",
    title = "Axes and named instances fall within correct ranges?",
    rationale = "According to the Open-Type spec's registered design-variation tags, instances in a variable font should have certain prescribed values.
        If a variable font has a 'wght' (Weight) axis, the valid coordinate range is 1-1000.
        If a variable font has a 'wdth' (Width) axis, the valid numeric range is strictly greater than zero.
        If a variable font has a 'slnt' (Slant) axis, then the coordinate of its 'Regular' instance is required to be 0.
        If a variable font has a 'ital' (Slant) axis, then the coordinate of its 'Regular' instance is required to be 0.",
    proposal = "https://github.com/fonttools/fontbakery/issues/2572"
)]
fn regular_coords_correct(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.is_variable_font(), "not-variable", "Not a variable font");
    let mut problems = vec![];
    if let Some(regular_location) = find_regular(f) {
        for (axis, expected) in REGULAR_COORDINATE_EXPECTATIONS {
            if let Some(actual) = regular_location.get(axis) {
                if *actual != expected {
                    problems.push(Status::fail(
                        &format!("{axis}-not-{expected:0}"),
                        &format!(
                            "Regular instance has {} coordinate of {}, expected {}",
                            axis, actual, expected
                        ),
                    ));
                }
            }
        }

        if let Some(actual) = regular_location.get("opsz") {
            if !(10.0..16.0).contains(actual) {
                problems.push(Status::warn(
                    "opsz",
                    &format!(
                        "Regular instance has opsz coordinate of {}, expected between 10 and 16",
                        actual
                    ),
                ));
            }
        }
    } else {
        return Ok(Status::just_one_fail(
            "no-regular-instance",
            "\"Regular\" instance not present.",
        ));
    }
    return_result(problems)
}
