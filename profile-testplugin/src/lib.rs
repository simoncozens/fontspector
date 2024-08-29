#![deny(clippy::unwrap_used, clippy::expect_used)]
use fontspector_checkapi::prelude::*;

struct Test;

#[check(
    id = "com.google.fonts/check/test/say_hello",
    title = "Check that the plugin protocol is working",
    rationale = "This check is part of the example of how to create plugins.",
    proposal = "https://github.com/simoncozens/fontspector/commit/5fdf9750991176c8e2776557ce6c17c642c24a73"
)]
fn say_hello(_c: &Testable, context: &Context) -> CheckFnResult {
    println!("Hello from the test plugin!");
    println!("My context was: {:?}", context);
    return_result(vec![])
}

#[check(
    id = "com.google.fonts/check/test/validate_toml",
    title = "Check that the filetype plugin protocol is working",
    rationale = "This check is part of the example of how to create plugins.",
    proposal = "https://github.com/simoncozens/fontspector/commit/5fdf9750991176c8e2776557ce6c17c642c24a73",
    applies_to = "TOML"
)]
fn validate_toml(c: &Testable, _context: &Context) -> CheckFnResult {
    let toml = std::fs::read_to_string(&c.filename)
        .map_err(|_| CheckError::Error("Couldn't open file".to_string()))?;
    Ok(if toml::from_str::<toml::Value>(&toml).is_ok() {
        Status::just_one_pass()
    } else {
        Status::just_one_fail("invalid-toml", "Invalid TOML")
    })
}

impl fontspector_checkapi::Plugin for Test {
    fn register(&self, cr: &mut Registry) -> Result<(), String> {
        let toml = FileType::new("*.toml");
        cr.register_filetype("TOML", toml);

        cr.register_simple_profile("test", vec![validate_toml, say_hello])
    }
}

#[cfg(not(target_family = "wasm"))]
pluginator::plugin_implementation!(fontspector_checkapi::Plugin, Test);
