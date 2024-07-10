use fontspector_checkapi::{return_result, Check, CheckRegistry, StatusList, TestFont};

struct Test;

fn say_hello(c: &TestFont) -> StatusList {
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
    fn provide_checks(&self, cr: &mut CheckRegistry) {
        cr.checks.push(SAY_HELLO)
    }
}
pluginator::plugin_implementation!(fontspector_checkapi::Plugin, Test);
