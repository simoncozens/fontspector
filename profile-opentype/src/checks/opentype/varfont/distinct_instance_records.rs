use std::collections::HashSet;

use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use skrifa::MetadataProvider;

#[check(
    id = "opentype/varfont/distinct_instance_records",
    title = "Validates that all of the instance records in a given font have distinct data",
    rationale = "According to the 'fvar' documentation in OpenType spec v1.9
        https://docs.microsoft.com/en-us/typography/opentype/spec/fvar

        All of the instance records in a font should have distinct coordinates
        and distinct subfamilyNameID and postScriptName ID values. If two or more
        records share the same coordinates, the same nameID values or the same
        postScriptNameID values, then all but the first can be ignored.",
    proposal = "https://github.com/fonttools/fontbakery/issues/3706"
)]
fn distinct_instance_records(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.is_variable_font(), "not-variable", "Not a variable font");

    let mut problems = vec![];
    let mut unique_records = HashSet::new();
    // We want to get at subfamily and postscript name IDs, so we use the lower-level
    // Skrifa API here.
    for instance in f.font().named_instances().iter() {
        let loc = instance.location();
        let coords: Vec<_> = loc.coords().to_vec();
        let subfamily_name_id = instance.subfamily_name_id();
        let postscript_name_id = instance.postscript_name_id();
        let instance_data = (coords.clone(), subfamily_name_id, postscript_name_id);
        if unique_records.contains(&instance_data) {
            let subfamily = f
                .get_name_entry_strings(subfamily_name_id)
                .next()
                .unwrap_or_else(|| format!("ID {}", subfamily_name_id));
            problems.push(Status::warn(
                &format!("repeated-instance-record:{subfamily}"),
                &format!(
                    "Instance {} with coordinates {:?} is duplicated",
                    subfamily, coords
                ),
            ));
        } else {
            unique_records.insert(instance_data);
        }
    }
    return_result(problems)
}
