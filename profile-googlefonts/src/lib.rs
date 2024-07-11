#![allow(renamed_and_removed_lints)]
include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));

use fonts_public::FamilyProto;

use fontspector_checkapi::prelude::*;

fn validate_metadatapb(c: &Testable) -> StatusList {
    let mdpb = std::fs::read_to_string(&c.filename).expect("Couldn't open file");
    if let Err(error) = protobuf::text_format::parse_from_str::<FamilyProto>(&mdpb) {
        Status::just_one_fail(&format!("Invalid METADATA.pb: {}", error))
    } else {
        Status::just_one_pass()
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

struct GoogleFonts;
impl fontspector_checkapi::Plugin for GoogleFonts {
    fn register(&self, cr: &mut Registry) {
        let mdpb = FileType::new("METADATA.pb");
        cr.register_filetype("MDPB", mdpb);

        cr.register_simple_profile("googlefonts", vec![VALIDATE_METADATA_PB]);
    }
}
pluginator::plugin_implementation!(fontspector_checkapi::Plugin, GoogleFonts);
