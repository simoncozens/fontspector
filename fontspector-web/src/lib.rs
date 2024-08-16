use js_sys::{Reflect, Uint8Array};
use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;
use fontspector_checkapi::{Check, CheckResult, Context, Plugin, Registry, Testable};
use profile_googlefonts::GoogleFonts;
use profile_universal::Universal;

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen]
pub fn check_fonts(fonts: &JsValue) -> Result<String, JsValue> {
    let mut registry = Registry::new();
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
                filename: filename.as_string().unwrap(),
                source: None,
                contents,
            }
        })
        .collect();
    let profile = registry.get_profile("googlefonts").unwrap();
    let context = Context {
        skip_network: true,
        network_timeout: None,
        configuration: serde_json::Map::new(),
    };

    let checkorder: Vec<(String, &Testable, &Check, Context)> = profile
        .sections
        .iter()
        .flat_map(|(sectionname, checknames)| {
            #[allow(clippy::unwrap_used)] // We previously ensured the check exists in the registry
            checknames
                .iter()
                // .filter(|checkname| included_excluded(checkname, &args))
                .map(|checkname| {
                    (
                        sectionname.clone(),
                        registry.checks.get(checkname).unwrap(),
                        context.clone(),
                    )
                })
        })
        .flat_map(|(sectionname, check, context): (String, &Check, Context)| {
            testables
                .iter()
                .filter(|testable| check.applies(testable, &registry))
                .map(move |testable| (sectionname.clone(), testable, check, context.clone()))
        })
        .collect();

    let results: Vec<CheckResult> = checkorder
        .iter()
        .map(|(sectionname, testable, check, context)| {
            (
                testable,
                check,
                check.run_one(testable, context, sectionname),
            )
        })
        .flat_map(|(_, _, result)| result)
        .collect();
    serde_json::to_string(&results).map_err(|e| e.to_string().into())
}
