use crate::checks::googlefonts::metadata::family_proto;
use fontspector_checkapi::prelude::*;

#[check(
    id = "googlefonts/metadata/category",
    rationale = "
        
        There are only five acceptable values for the category field in a METADATA.pb
        file:

        - MONOSPACE

        - SANS_SERIF

        - SERIF

        - DISPLAY

        - HANDWRITING

        This check is meant to avoid typos in this field.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/2972",
    title = "Ensure METADATA.pb category field is valid.",
    implementation = "all"
)]
fn category(c: &TestableCollection, _context: &Context) -> CheckFnResult {
    let mdpb = c
        .get_file("METADATA.pb")
        .ok_or_else(|| CheckError::skip("no-mdpb", "No METADATA.pb file found"))?;
    let family_metadata = family_proto(mdpb)?;
    let mut problems = vec![];
    for category_value in family_metadata.category {
        if vec!["MONOSPACE", "SANS_SERIF", "SERIF", "DISPLAY", "HANDWRITING"]
            .iter()
            .all(|s| category_value != *s.to_string())
        {
            problems.push(Status::fail(
                "bad-value",
                &format!(
                    "The field category has \"{}\" which is not valid.",
                    category_value
                ),
            ));
        }
    }
    return_result(problems)
}
