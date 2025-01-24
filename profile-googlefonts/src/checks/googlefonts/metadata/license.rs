use crate::checks::googlefonts::metadata::family_proto;
use fontspector_checkapi::prelude::*;

#[check(
    id="googlefonts/metadata/license",
    rationale="
        The license field in METADATA.pb must contain one of the
        three values \"APACHE2\", \"UFL\" or \"OFL\". (New fonts should
        generally be OFL unless there are special circumstances.)
    ",
    applies_to = "MDPB",
    proposal="https://github.com/fonttools/fontbakery/issues/4829",  // legacy check
    title="METADATA.pb license is \"APACHE2\", \"UFL\" or \"OFL\"?"
)]
fn license(c: &Testable, _context: &Context) -> CheckFnResult {
    let msg = family_proto(c).map_err(|e| {
        CheckError::Error(format!("METADATA.pb is not a valid FamilyProto: {:?}", e))
    })?;
    if msg.license() != "APACHE2" && msg.license() != "UFL" && msg.license() != "OFL" {
        Ok(Status::just_one_fail(
            "bad-license",
            &format!(
                "'METADATA.pb license field (\"{}\") must be \"APACHE2\", \"UFL\" or \"OFL\".",
                msg.license()
            ),
        ))
    } else {
        Ok(Status::just_one_pass())
    }
}
