#![deny(clippy::unwrap_used, clippy::expect_used)]
pub mod constants;
mod description;
mod family;
mod license;
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
        cr.register_check(license::name_rfn);
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
"googlefonts/description/broken_links",
"googlefonts/description/eof_linebreak",
"googlefonts/description/family_update",
"googlefonts/description/git_url",
"googlefonts/description/has_article",
"googlefonts/description/has_unsupported_elements",
"googlefonts/description/min_length",
"googlefonts/description/urls",
"googlefonts/description/valid_html",
]
"Licensing Checks" = [
"googlefonts/family/has_license",
"googlefonts/font_copyright",
"googlefonts/license/OFL_body_text",
"googlefonts/license/OFL_copyright",
"googlefonts/metadata/copyright",
"googlefonts/metadata/license",
"googlefonts/metadata/reserved_font_name",
"googlefonts/name/license",
"googlefonts/name/license_url",
"googlefonts/name/rfn",
]
"#,
        )
        .map_err(|_| "Couldn't parse profile")?;

        cr.register_profile("googlefonts", profile)
    }
}
