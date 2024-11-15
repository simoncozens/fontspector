#![allow(renamed_and_removed_lints, clippy::unwrap_used)]
include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
use fonts_public::FamilyProto;
use fontspector_checkapi::{prelude::*, StatusCode};

fn family_proto(t: &Testable) -> Result<FamilyProto, CheckError> {
    let mdpb = std::str::from_utf8(&t.contents)
        .map_err(|_| CheckError::Error("METADATA.pb is not valid UTF-8".to_string()))?;
    protobuf::text_format::parse_from_str::<FamilyProto>(mdpb)
        .map_err(|e| CheckError::Error(format!("Error parsing METADATA.pb: {}", e)))
}

#[check(
    id="googlefonts/metadata/copyright",
    rationale="
        The METADATA.pb file includes a copyright field for each font
        file in the family. The value of this field should be the same
        for all fonts in the family.
    ",
    applies_to = "MDPB",
    proposal="https://github.com/fonttools/fontbakery/issues/4829",  // legacy check
    title="METADATA.pb: Copyright notice is the same in all fonts?"
)]
fn metadata_copyright(c: &Testable, context: &Context) -> CheckFnResult {
    let msg = family_proto(c).map_err(|e| {
        CheckError::Error(format!("METADATA.pb is not a valid FamilyProto: {:?}", e))
    })?;
    assert_all_the_same( 
        context,
        &(msg.fonts.iter().map(|f| 
            (f.copyright(),
            f.copyright(),
            f.filename())
        ).collect::<Vec<_>>()),
        "inconsistency",
        "METADATA.pb: Copyright field value is inconsistent across the family.\nThe following copyright values were found:",
        StatusCode::Fail,
    )
}
