#![deny(clippy::unwrap_used, clippy::expect_used)]
mod checks;
use fontspector_checkapi::{Profile, Registry};

pub struct Universal;

impl fontspector_checkapi::Plugin for Universal {
    fn register(&self, cr: &mut Registry) -> Result<(), String> {
        cr.register_check(checks::CHECK_NAME_TRAILING_SPACES);
        cr.register_check(checks::CHECK_UNWANTED_TABLES);
        cr.register_check(checks::CHECK_REQUIRED_TABLES);
        let profile = Profile::from_toml(
            r#"
[sections]
"Universal Profile Checks" = [
"com.google.fonts/check/name/trailing_spaces",
"com.google.fonts/check/unwanted_tables",
"com.google.fonts/check/required_tables",
]
"#,
        )
        .map_err(|_| "Couldn't parse profile")?;
        cr.register_profile("universal", profile)
    }
}
