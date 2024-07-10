use fontspector_checkapi::{return_result, Check, Profile, Registry, StatusList, TestFont};

struct Test;

fn say_hello(_c: &TestFont) -> StatusList {
    println!("Hello from the test plugin!");
    return_result(vec![])
}

pub const SAY_HELLO: Check = Check {
    id: "com.google.fonts/check/test/say_hello",
    title: "Check that the plugin protocol is working",
    rationale: None,
    proposal: None,
    check_all: None,
    check_one: Some(&say_hello),
};

impl fontspector_checkapi::Plugin for Test {
    fn register(&self, cr: &mut Registry) {
        cr.checks.push(SAY_HELLO);
        cr.register_profile(
            "test",
            Profile::from_toml(
                r#"
[sections]
"A test profile" = [
    "com.google.fonts/check/test/say_hello"
]
"#,
            )
            .expect("Couldn't parse profile"),
        )
    }
}
pluginator::plugin_implementation!(fontspector_checkapi::Plugin, Test);
