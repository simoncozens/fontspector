use std::collections::HashSet;

use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use skrifa::MetadataProvider;

#[check(
    id = "opentype/varfont/same_size_instance_records",
    title = "Validates that all of the instance records in a given font have the same size",
    rationale = "According to the 'fvar' documentation in OpenType spec v1.9
        https://docs.microsoft.com/en-us/typography/opentype/spec/fvar

        All of the instance records in a given font must be the same size, with
        all either including or omitting the postScriptNameID field. [...]
        If the value is 0xFFFF, then the value is ignored, and no PostScript name
        equivalent is provided for the instance.",
    proposal = "https://github.com/fonttools/fontbakery/issues/3705"
)]
fn same_size_instance_records(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.is_variable_font(), "not-variable", "Not a variable font");
    skip!(
        f.font().named_instances().is_empty(),
        "no-instance-records",
        "Font has no instance records."
    );
    let has_or_hasnt_postscriptname: HashSet<bool> = f
        .font()
        .named_instances()
        .iter()
        .map(|ni| ni.postscript_name_id().is_none())
        .collect();
    Ok(if has_or_hasnt_postscriptname.len() > 1 {
        Status::just_one_fail(
            "different-size-instance-records",
            "Instance records don't all have the same size.",
        )
    } else {
        Status::just_one_pass()
    })
}
