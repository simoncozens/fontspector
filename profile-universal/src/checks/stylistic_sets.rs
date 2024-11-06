use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};

#[check(
    id = "stylisticset_description",
    rationale = "
        Stylistic sets should provide description text. Programs such as InDesign,
        TextEdit and Inkscape use that info to display to the users so that they know
        what a given stylistic set offers.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3155",
    title = "Ensure Stylistic Sets have description."
)]
fn stylisticset_description(f: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    let mut problems = vec![];
    for (feature_record, feature) in font.feature_records(true) {
        let tag = feature_record.feature_tag().to_string();
        if tag.starts_with("ss") && feature?.feature_params().is_none() {
            problems.push(Status::warn(
                "missing-description",
                &format!(
                    "The stylistic set {} lacks a description string in the name table",
                    tag
                ),
            ));
        }
    }
    return_result(problems)
}
