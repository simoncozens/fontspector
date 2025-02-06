use crate::checks::googlefonts::metadata::family_proto;
use fontspector_checkapi::{prelude::*, StatusCode};

#[check(
    id = "googlefonts/metadata/familyname",
    rationale = "
        
        The METADATA.pb file includes a family name field for each font
        file in the family. The value of this field should be the same
        for all fonts in the family.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Check that METADATA.pb family values are all the same.",
    implementation = "all"
)]
fn familyname(c: &TestableCollection, context: &Context) -> CheckFnResult {
    let mdpb = c
        .get_file("METADATA.pb")
        .ok_or_else(|| CheckError::skip("no-mdpb", "No METADATA.pb file found"))?;
    let msg = family_proto(mdpb)?;
    assert_all_the_same(
        context,
        &(msg.fonts.iter().map(|f|
            (f.name(),
            f.name(),
            f.filename())
        ).collect::<Vec<_>>()),
        "inconsistency",
        "METADATA.pb: family name value is inconsistent across the family.\nThe following name values were found:",
        StatusCode::Fail,
    )
}
