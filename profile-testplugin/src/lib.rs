#![deny(clippy::unwrap_used, clippy::expect_used)]
use fontspector_checkapi::prelude::*;

struct Test;

fn say_hello(_c: &Testable, context: &Context) -> CheckFnResult {
    println!("Hello from the test plugin!");
    println!("My context was: {:?}", context);
    return_result(vec![])
}

fn validate_toml(c: &Testable, _context: &Context) -> CheckFnResult {
    let toml = std::fs::read_to_string(&c.filename).map_err(|_| "Couldn't open file")?;
    Ok(if toml::from_str::<toml::Value>(&toml).is_ok() {
        Status::just_one_pass()
    } else {
        Status::just_one_fail("Invalid TOML")
    })
}
pub const SAY_HELLO: Check = Check {
    id: "com.google.fonts/check/test/say_hello",
    title: "Check that the plugin protocol is working",
    rationale: "This check is part of the example of how to create plugins.",
    proposal: "https://github.com/simoncozens/fontspector/commit/5fdf9750991176c8e2776557ce6c17c642c24a73",
    check_all: None,
    check_one: Some(&say_hello),
    applies_to: "TTF",
    hotfix: None,
    fix_source: None,
};

pub const VALIDATE_TOML: Check = Check {
    id: "com.google.fonts/check/test/validate_toml",
    title: "Check that the filetype plugin protocol is working",
    rationale: "This check is part of the example of how to create plugins.",
    proposal: "https://github.com/simoncozens/fontspector/commit/5fdf9750991176c8e2776557ce6c17c642c24a73",
    check_all: None,
    check_one: Some(&validate_toml),
    applies_to: "TOML",
    hotfix: None,
    fix_source: None,
};

impl fontspector_checkapi::Plugin for Test {
    fn register(&self, cr: &mut Registry) -> Result<(), String> {
        let toml = FileType::new("*.toml");
        cr.register_filetype("TOML", toml);

        cr.register_simple_profile("test", vec![VALIDATE_TOML, SAY_HELLO])
    }
}
pluginator::plugin_implementation!(fontspector_checkapi::Plugin, Test);
