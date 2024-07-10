mod checks;
use fontspector_checkapi::{Profile, Registry};

pub struct Universal;

impl fontspector_checkapi::Plugin for Universal {
    fn register(&self, cr: &mut Registry) {
        cr.checks.push(checks::BOLD_ITALIC_UNIQUE_CHECK);
        cr.checks.push(checks::NAME_TRAILING_SPACES_CHECK);
        cr.checks.push(checks::UNWANTED_TABLES_CHECK);
        cr.register_profile(
            "universal",
            Profile::from_toml(
                r#"
[sections]
"OpenType Specification Checks" = [
    "com.adobe.fonts/check/family/bold_italic_unique_for_nameid1"
]
"Universal Profile Checks" = [
    "com.google.fonts/check/name/trailing_spaces",
    "com.google.fonts/check/unwanted_tables"
]
"#,
            )
            .expect("Couldn't parse profile"),
        )
    }
}
pluginator::plugin_implementation!(fontspector_checkapi::Plugin, Universal);
