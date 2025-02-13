use crate::checks::googlefonts::metadata::family_proto;
use fontspector_checkapi::prelude::*;

#[check(
    id = "googlefonts/metadata/has_regular",
    rationale = "
        
        According to Google Fonts standards, families should have a Regular
        style.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Ensure there is a regular style defined in METADATA.pb.",
    implementation = "all"
)]
fn has_regular(c: &TestableCollection, _context: &Context) -> CheckFnResult {
    let mdpb = c
        .get_file("METADATA.pb")
        .ok_or_else(|| CheckError::skip("no-mdpb", "No METADATA.pb file found"))?;
    let msg = family_proto(mdpb)?;
    if msg
        .fonts
        .iter()
        .any(|f| f.weight() == 400 && f.style() == "normal")
    {
        Ok(Status::just_one_pass())
    } else {
        Ok(Status::just_one_fail(
        "lacks-regular",
        "This family lacks a Regular (style: normal and weight: 400) as required by Google Fonts standards. If family consists of a single-weight non-Regular style only, consider the Google Fonts specs for this case: https://github.com/googlefonts/gf-docs/tree/main/Spec#single-weight-families"
        ))
    }
}
