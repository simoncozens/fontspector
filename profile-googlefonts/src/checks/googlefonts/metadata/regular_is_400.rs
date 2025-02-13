use crate::checks::googlefonts::metadata::family_proto;
use fontspector_checkapi::prelude::*;

#[check(
    id = "googlefonts/metadata/regular_is_400",
    rationale = "
        
        The weight of the regular style should be set to 400.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "METADATA.pb: Regular should be 400.",
    implementation = "all"
)]
fn regular_is_400(c: &TestableCollection, context: &Context) -> CheckFnResult {
    let mdpb = c
        .get_file("METADATA.pb")
        .ok_or_else(|| CheckError::skip("no-mdpb", "No METADATA.pb file found"))?;
    let msg = family_proto(mdpb)?;
    let badfonts = msg
        .fonts
        .iter()
        .filter(|f| f.full_name().starts_with("Regular") && f.weight() != 400)
        .map(|f| f.filename().to_string())
        .collect::<Vec<_>>();

    if badfonts.is_empty() {
        Ok(Status::just_one_pass())
    } else {
        Ok(Status::just_one_fail(
            "not-400",
            &format!(
                "METADATA.pb: Regular font weight must be 400. Please fix these: {}",
                bullet_list(context, badfonts),
            ),
        ))
    }
}
