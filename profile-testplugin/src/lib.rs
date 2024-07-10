use fontspector_checkapi::prelude::*;

struct Test;

fn say_hello(_c: &Testable) -> StatusList {
    println!("Hello from the test plugin!");
    return_result(vec![])
}

fn validate_toml(c: &Testable) -> StatusList {
    let toml = std::fs::read_to_string(&c.filename).expect("Couldn't open file");
    if toml::from_str::<toml::Value>(&toml).is_ok() {
        Status::just_one_pass()
    } else {
        Status::just_one_fail("Invalid TOML")
    }
}
pub const SAY_HELLO: Check = Check {
    id: "com.google.fonts/check/test/say_hello",
    title: "Check that the plugin protocol is working",
    rationale: None,
    proposal: None,
    check_all: None,
    check_one: Some(&say_hello),
    applies_to: "TTF",
};

pub const VALIDATE_TOML: Check = Check {
    id: "com.google.fonts/check/test/validate_toml",
    title: "Check that the filetype plugin protocol is working",
    rationale: None,
    proposal: None,
    check_all: None,
    check_one: Some(&validate_toml),
    applies_to: "TOML",
};

impl fontspector_checkapi::Plugin for Test {
    fn register(&self, cr: &mut Registry) {
        cr.checks.push(SAY_HELLO);
        cr.checks.push(VALIDATE_TOML);
        let toml = FileType::new("*.toml");
        cr.register_filetype("TOML", toml);

        cr.register_profile(
            "test",
            Profile::from_toml(
                r#"
[sections]
"A test profile" = [
    "com.google.fonts/check/test/say_hello",
    "com.google.fonts/check/test/validate_toml"
]
"#,
            )
            .expect("Couldn't parse profile"),
        )
    }
}
pluginator::plugin_implementation!(fontspector_checkapi::Plugin, Test);
