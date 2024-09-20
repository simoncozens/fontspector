#![deny(clippy::unwrap_used, clippy::expect_used)]
mod description;
mod family;
mod metadata;
use fontspector_checkapi::prelude::*;

pub struct GoogleFonts;
impl fontspector_checkapi::Plugin for GoogleFonts {
    fn register(&self, cr: &mut Registry) -> Result<(), String> {
        let mdpb = FileType::new("METADATA.pb");
        let desc = FileType::new("DESCRIPTION.en_us.html");
        cr.register_filetype("MDPB", mdpb);
        cr.register_filetype("DESC", desc);
        cr.register_check(description::description_min_length);
        cr.register_check(description::description_eof_linebreak);
        cr.register_check(family::family_equal_codepoint_coverage);
        cr.register_check(metadata::validate_metadatapb);
        cr.register_check(metadata::can_render_samples);
        let profile = Profile::from_toml(
            r#"
include_profiles = ["universal"]
[sections]
"Metadata Checks" = [
"googlefonts/metadata/parses",
"googlefonts/metadata/can_render_samples",
]
"Family Checks" = [
"googlefonts/family/equal_codepoint_coverage",
]
"Description Checks" = [
"googlefonts/description/min_length",
"googlefonts/description/eof_linebreak",
]
"#,
        )
        .map_err(|_| "Couldn't parse profile")?;

        cr.register_profile("googlefonts", profile)
    }
}
