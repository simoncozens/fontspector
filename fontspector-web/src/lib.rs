use js_sys::{Reflect, Uint8Array};
use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;
use fontspector_checkapi::{
    Check, CheckResult, Context, Plugin, Registry, Testable, TestableCollection, TestableType,
};
use profile_googlefonts::GoogleFonts;
use profile_opentype::OpenType;
use profile_universal::Universal;

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen]
pub fn check_fonts(fonts: &JsValue, profile: &str) -> Result<String, JsValue> {
    console_error_panic_hook::set_once();

    let mut registry = Registry::new();
    OpenType
        .register(&mut registry)
        .expect("Couldn't register opentype profile, fontspector bug");
    Universal
        .register(&mut registry)
        .expect("Couldn't register universal profile, fontspector bug");
    GoogleFonts
        .register(&mut registry)
        .expect("Couldn't register googlefonts profile, fontspector bug");
    let testables: Vec<Testable> = Reflect::own_keys(fonts)?
        .into_iter()
        .map(|filename| {
            let file: JsValue = Reflect::get(fonts, &filename).unwrap();
            let contents = Uint8Array::new(&file).to_vec();

            Testable {
                filename: filename.as_string().unwrap().into(),
                source: None,
                contents,
            }
        })
        .collect();
    let collection = TestableCollection::from_testables(testables);

    let profile = registry
        .get_profile(profile)
        .ok_or_else(|| format!("Could not find profile {:?}", profile))?;
    let context = Context {
        skip_network: true,
        network_timeout: None,
        configuration: serde_json::Map::new(),
        check_metadata: serde_json::Value::Null,
        full_lists: false,
        cache: Default::default(),
    };
    let all_testables: Vec<TestableType> = collection.collection_and_files().collect();

    let checkorder: Vec<(String, &TestableType, &Check, Context)> = profile.check_order(
        &None,
        &None,
        &registry,
        context,
        serde_json::Map::new(),
        &all_testables,
    );

    let results: Vec<CheckResult> = checkorder
        .iter()
        .map(|(sectionname, testable, check, context)| {
            (
                testable,
                check,
                check.run(testable, context, Some(sectionname)),
            )
        })
        .flat_map(|(_, _, result)| result)
        .collect();
    serde_json::to_string(&results).map_err(|e| e.to_string().into())
}
