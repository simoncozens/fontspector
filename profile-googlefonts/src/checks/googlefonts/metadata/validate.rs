#![allow(renamed_and_removed_lints, clippy::unwrap_used)]

use crate::checks::googlefonts::metadata::family_proto;
use chrono::prelude::*;
use fontspector_checkapi::prelude::*;

#[check(
    id = "googlefonts/metadata/validate",
    title = "Check METADATA.pb parses correctly",
    rationale = "
        The purpose of this check is to ensure that the METADATA.pb file is not
        malformed.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/2248",
    applies_to = "MDPB"
)]
fn validate(c: &Testable, _context: &Context) -> CheckFnResult {
    let msg = family_proto(c).map_err(|e| {
        CheckError::Error(format!("METADATA.pb is not a valid FamilyProto: {:?}", e))
    })?;
    let mut problems = vec![];
    if let Some(designer) = msg.designer.as_ref() {
        if designer.contains('/') {
            problems.push(Status::fail("slash",
                    &format!(
                    "Font designer field contains a forward slash '{}'. Please use commas to separate multiple names instead.",
                    designer
                )));
        }
    }

    // Check date added is YYYY-MM-DD
    if msg
        .date_added
        .as_ref()
        .is_some_and(|da| NaiveDate::parse_from_str(da, "%Y-%m-%d").is_err())
    {
        problems.push(Status::fail(
            "date-malformed",
            "Date added is not in the format YYYY-MM-DD",
        ))
    }
    return_result(problems)
}
