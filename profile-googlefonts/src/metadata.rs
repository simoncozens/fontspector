#![allow(renamed_and_removed_lints, clippy::unwrap_used)]
include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));

use chrono::prelude::*;
use fonts_public::FamilyProto;
use fontspector_checkapi::prelude::*;

fn validate_metadatapb(c: &Testable) -> CheckFnResult {
    let mdpb =
        std::fs::read_to_string(&c.filename).map_err(|_| "Couldn't open file".to_string())?;
    match protobuf::text_format::parse_from_str::<FamilyProto>(&mdpb) {
        Err(error) => Ok(Status::just_one_fail(&format!(
            "Invalid METADATA.pb: {}",
            error
        ))),
        Ok(msg) => {
            let mut problems = vec![];
            if let Some(designer) = msg.designer.as_ref() {
                if designer.contains('/') {
                    problems.push(Status::fail(&format!(
                    "Font designer field contains a forward slash '{}'. Please use commas to separate multiple names instead.",
                    designer
                )));
                }
            }
            // Check subsets are in order
            let mut sorted_subsets = msg.subsets.clone();
            sorted_subsets.sort();
            if msg.subsets != sorted_subsets {
                problems.push(Status::fail("Subsets are not in order"))
            }

            // Check date added is YYYY-MM-DD
            if msg
                .date_added
                .as_ref()
                .is_some_and(|da| NaiveDate::parse_from_str(da, "%Y-%m-%d").is_err())
            {
                problems.push(Status::fail("Date added is not in the format YYYY-MM-DD"))
            }
            return_result(problems)
        }
    }
}

pub const CHECK_METADATA_PARSES: Check = Check {
    id: "com.google.fonts/check/metadata/parses",
    title: "Check METADATA.pb parses correctly",
    rationale: "
        The purpose of this check is to ensure that the METADATA.pb file is not
        malformed.
    ",
    proposal: "https://github.com/fonttools/fontbakery/issues/2248",
    check_all: None,
    check_one: Some(&validate_metadatapb),
    applies_to: "MDPB",
    hotfix: None,
    fix_source: None,
};
