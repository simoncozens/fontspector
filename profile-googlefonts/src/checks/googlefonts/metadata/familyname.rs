use crate::checks::googlefonts::metadata::family_proto;
use fontspector_checkapi::prelude::*;
use std::collections::HashMap;

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
fn familyname(c: &TestableCollection, _context: &Context) -> CheckFnResult {
    let mdpb = c
        .get_file("METADATA.pb")
        .ok_or_else(|| CheckError::skip("no-mdpb", "No METADATA.pb file found"))?;
    let family_metadata = family_proto(mdpb)?;

    let mut names: HashMap<String, String> = HashMap::new();
    for f in family_metadata.fonts {
        #[allow(clippy::unwrap_used)]
        if f.name.is_some() && f.filename.is_some() {
            names.insert(f.name.unwrap(), f.filename.unwrap());
        }
    }
    //let mut names: HashMap<String, String> = family_metadata.fonts
    //    .iter()
    //    .map(|f| (f.name.to_string(), f.filename.to_string()))
    //    .collect();
    if names.len() > 1 {
        return Ok(Status::just_one_fail(
            "inconsistency",
            "METADATA.pb: family name value is inconsistent across the family.\n", // TODO: "The following name values were found:\n\n"
                                                                                   // TODO: + show_inconsistencies(names, config),
        ));
    }
    Ok(Status::just_one_pass())
}
