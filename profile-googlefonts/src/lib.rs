#![deny(clippy::unwrap_used, clippy::expect_used)]
mod family;
mod metadata;
use family::CHECK_FAMILY_EQUAL_CODEPOINT_COVERAGE;
use fontspector_checkapi::prelude::*;
use metadata::CHECK_METADATA_PARSES;

pub struct GoogleFonts;
impl fontspector_checkapi::Plugin for GoogleFonts {
    fn register(&self, cr: &mut Registry) -> Result<(), String> {
        let mdpb = FileType::new("METADATA.pb");
        cr.register_filetype("MDPB", mdpb);
        cr.register_check(CHECK_FAMILY_EQUAL_CODEPOINT_COVERAGE);
        cr.register_check(CHECK_METADATA_PARSES);
        let profile = Profile::from_toml(
            r#"
include_profiles = ["universal"]
[sections]
"Metadata Checks" = [
"com.google.fonts/check/metadata/parses",
]
"Family Checks" = [
"com.google.fonts/check/family/equal_codepoint_coverage"
]
"#,
        )
        .map_err(|_| "Couldn't parse profile")?;

        cr.register_profile("googlefonts", profile)
    }
}
