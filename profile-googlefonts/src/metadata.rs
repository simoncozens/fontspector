#![allow(renamed_and_removed_lints, clippy::unwrap_used)]
include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));

use chrono::prelude::*;
use fonts_public::FamilyProto;
use fontspector_checkapi::prelude::*;

#[check(
    id = "com.google.fonts/check/metadata/parses",
    title = "Check METADATA.pb parses correctly",
    rationale = "
        The purpose of this check is to ensure that the METADATA.pb file is not
        malformed.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/2248",
    applies_to = "MDPB"
)]
fn validate_metadatapb(c: &Testable, _context: &Context) -> CheckFnResult {
    let mdpb = std::str::from_utf8(&c.contents)
        .map_err(|_| CheckError::Error("METADATA.pb is not valid UTF-8".to_string()))?;
    let msg = protobuf::text_format::parse_from_str::<FamilyProto>(mdpb)
        .map_err(|e| CheckError::Error(format!("Error parsing METADATA.pb: {}", e)))?;
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
    // Check subsets are in order
    let mut sorted_subsets = msg.subsets.clone();
    sorted_subsets.sort();
    if msg.subsets != sorted_subsets {
        problems.push(Status::fail("not-sorted", "Subsets are not in order"))
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
