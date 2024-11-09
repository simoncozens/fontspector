use std::collections::HashMap;

use fontspector_checkapi::{prelude::*, skip, FileTypeConvert};
use skrifa::MetadataProvider;

#[check(
    id = "varfont/consistent_axes",
    rationale = "
        In order to facilitate the construction of intuitive and friendly user
        interfaces, all variable font files in a given family should have the same set
        of variation axes. Also, each axis must have a consistent setting of min/max
        value ranges accross all the files.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/2810",
    title = "Ensure that all variable font files have the same set of axes and axis ranges.",
    implementation = "all"
)]
fn consistent_axes(c: &TestableCollection, _context: &Context) -> CheckFnResult {
    let ttfs = TTF.from_collection(c);
    let fonts = ttfs
        .iter()
        .filter(|f| f.is_variable_font())
        .collect::<Vec<_>>();
    let mut problems = vec![];
    skip!(fonts.len() < 2, "no-siblings", "No sibling fonts found");
    let mut reference_ranges = HashMap::new();
    for font in fonts.iter() {
        for axis in font.font().axes().iter() {
            reference_ranges
                .entry(axis.tag())
                .or_insert((axis.min_value(), axis.max_value()));
        }
    }
    for font in fonts.iter() {
        for (axis, &(a_min, a_max)) in reference_ranges.iter() {
            if let Some(found_axis) = font.font().axes().iter().find(|a| a.tag() == *axis) {
                if found_axis.min_value() != a_min || found_axis.max_value() != a_max {
                    problems.push(Status::fail(
                        "inconsistent-axis-range",
                        &format!(
                            "Font {} has inconsistent range for axis {}: expected [{}, {}], found [{}, {}]",
                            font.filename.to_str().unwrap_or("Unknown font"),
                            axis,
                            a_min,
                            a_max,
                            found_axis.min_value(),
                            found_axis.max_value()
                        ),
                    ));
                }
            } else {
                problems.push(Status::fail(
                    "missing-axis",
                    &format!(
                        "Font {} is missing axis {}",
                        font.filename.to_str().unwrap_or("Unknown font"),
                        axis
                    ),
                ));
            }
        }
    }
    return_result(problems)
}
