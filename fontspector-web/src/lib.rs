use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;
use fontspector_checkapi::{Check, CheckResult, Context, Plugin, Registry, Testable};
use profile_googlefonts::GoogleFonts;
use profile_universal::Universal;

#[wasm_bindgen]
pub fn test(font_a: &[u8]) -> String {
    let mut registry = Registry::new();
    Universal
        .register(&mut registry)
        .expect("Couldn't register universal profile, fontspector bug");
    GoogleFonts
        .register(&mut registry)
        .expect("Couldn't register googlefonts profile, fontspector bug");
    let testable: Testable = Testable {
        filename: "font.ttf".to_string(),
        source: None,
        contents: font_a.to_vec(),
    };
    let testables = vec![testable];
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
    serde_json::to_string(&results).unwrap_or("Couldn't do it".to_string())
}
