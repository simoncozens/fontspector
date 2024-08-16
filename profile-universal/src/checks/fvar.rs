use std::{borrow::Cow, collections::HashMap, vec};

use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use skrifa::MetadataProvider;

fn regular_coords_correct(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    if !f.is_variable_font() {
        skip!("not-variable", "Not a variable font");
    }
    let axes = f.font().axes();
    let mut problems = vec![];
    let regular_instance = f
        .font()
        .named_instances()
        .iter()
        .find(|ni| {
            f.font()
                .localized_strings(ni.subfamily_name_id())
                .any(|s| "Regular" == s.chars().collect::<Cow<str>>())
        })
        .ok_or(CheckError::Skip {
            code: "no-regular".to_string(),
            message: "No Regular instance found".to_string(),
        })?;
    let regular_location: HashMap<String, f32> = regular_instance
        .user_coords()
        .zip(axes.iter())
        .map(|(coord, axis)| (axis.tag().to_string(), coord))
        .collect();
    let expectations = vec![("wght", 400.0), ("wdth", 100.0), ("slnt", 0.0)];
    for (axis, expected) in expectations {
        if let Some(actual) = regular_location.get(axis) {
            if *actual != expected {
                problems.push(Status::fail(
                    axis,
                    &format!(
                        "Regular instance has {} coordinate of {}, expected {}",
                        axis, actual, expected
                    ),
                ));
            }
        }
    }
    return_result(problems)
}

pub const CHECK_REGULAR_COORDS_CORRECT: Check = Check {
    id: "com.google.fonts/check/fvar/regular_coords_correct",
    title: "Regular instance coordinates are correct?",
    rationale: "According to the Open-Type spec's registered design-variation tags, the Regular instance in a variable font should have certain prescribed values.
        If a variable font has a 'wght' (Weight) axis, then the coordinate of its 'Regular' instance is required to be 400.
        If a variable font has a 'wdth' (Width) axis, then the coordinate of its 'Regular' instance is required to be 100.
        If a variable font has a 'slnt' (Slant) axis, then the coordinate of its 'Regular' instance is required to be 0.",
    proposal: "https://github.com/fonttools/fontbakery/issues/1707",
    check_one: Some(&regular_coords_correct),
    check_all: None,
    applies_to: "TTF",
    hotfix: None,
    fix_source: None,
    flags: CheckFlags::default(),
};
