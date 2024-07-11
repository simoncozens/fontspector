#![allow(renamed_and_removed_lints)]
include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));

use chrono::prelude::*;
use fonts_public::FamilyProto;
use fontspector_checkapi::prelude::*;

fn validate_metadatapb(c: &Testable) -> StatusList {
    let mdpb = std::fs::read_to_string(&c.filename).expect("Couldn't open file");
    match protobuf::text_format::parse_from_str::<FamilyProto>(&mdpb) {
        Err(error) => Status::just_one_fail(&format!("Invalid METADATA.pb: {}", error)),
        Ok(msg) => {
            let mut problems = vec![];
            if msg.designer.as_ref().is_some_and(|d| d.contains('/')) {
                problems.push(Status::fail(&format!(
                    "Font designer field contains a forward slash '{}'. Please use commas to separate multiple names instead.",
                    msg.designer.as_ref().unwrap()
                )))
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

pub const VALIDATE_METADATA_PB: Check = Check {
    id: "com.google.fonts/check/metadata/parses",
    title: "Check METADATA.pb parse correctly",
    rationale: None,
    proposal: None,
    check_all: None,
    check_one: Some(&validate_metadatapb),
    applies_to: "MDPB",
    hotfix: None,
    fix_source: None,
};
