mod checks;
use fontspector_checkapi::{Profile, Registry};

pub struct Universal;

impl fontspector_checkapi::Plugin for Universal {
    fn register(&self, cr: &mut Registry) -> Result<(), String> {
        cr.register_check(checks::NAME_TRAILING_SPACES_CHECK);
        cr.register_check(checks::UNWANTED_TABLES_CHECK);
        cr.register_check(checks::REQUIRED_TABLES_CHECK);
        cr.register_profile(
            "universal",
            Profile::from_toml(
                r#"
[sections]
"Universal Profile Checks" = [
    "com.google.fonts/check/name/trailing_spaces",
    "com.google.fonts/check/unwanted_tables",
    "com.google.fonts/check/required_tables",
]
"#,
            )
            .expect("Couldn't parse profile"),
        )
    }
}
pluginator::plugin_implementation!(fontspector_checkapi::Plugin, Universal);
