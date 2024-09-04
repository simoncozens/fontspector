#![deny(clippy::unwrap_used, clippy::expect_used)]
mod family;
mod metadata;
use fontspector_checkapi::prelude::*;

pub struct GoogleFonts;
impl fontspector_checkapi::Plugin for GoogleFonts {
    fn register(&self, cr: &mut Registry) -> Result<(), String> {
        let mdpb = FileType::new("METADATA.pb");
        cr.register_filetype("MDPB", mdpb);
        cr.register_check(family::family_equal_codepoint_coverage);
        cr.register_check(metadata::validate_metadatapb);
        cr.register_check(metadata::can_render_samples);
        let profile = Profile::from_toml(
            r#"
include_profiles = ["universal"]
[sections]
"Metadata Checks" = [
"com.google.fonts/check/metadata/parses",
"com.google.fonts/check/metadata/can_render_samples",
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
