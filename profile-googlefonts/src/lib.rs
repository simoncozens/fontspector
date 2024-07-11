mod metadata;
use fontspector_checkapi::prelude::*;
use metadata::VALIDATE_METADATA_PB;

struct GoogleFonts;
impl fontspector_checkapi::Plugin for GoogleFonts {
    fn register(&self, cr: &mut Registry) -> Result<(), String> {
        let mdpb = FileType::new("METADATA.pb");
        cr.register_filetype("MDPB", mdpb);
        cr.register_check(VALIDATE_METADATA_PB);

        cr.register_profile(
            "googlefonts",
            Profile::from_toml(
                r#"
include_profiles = ["universal"]
[sections]
"Metadata Checks" = [
    "com.google.fonts/check/metadata/parses",
]
"#,
            )
            .expect("Couldn't parse profile"),
        )
    }
}
pluginator::plugin_implementation!(fontspector_checkapi::Plugin, GoogleFonts);
