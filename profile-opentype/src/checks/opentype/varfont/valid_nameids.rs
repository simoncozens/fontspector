use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use read_fonts::types::NameId;
use skrifa::MetadataProvider;

#[check(
    id = "opentype/varfont/valid_nameids",
    title = "Validates that all of the name IDs in an instance record are within the correct range",
    rationale = r#"
        According to the 'fvar' documentation in OpenType spec v1.9
        https://docs.microsoft.com/en-us/typography/opentype/spec/fvar

        The axisNameID field provides a name ID that can be used to obtain strings
        from the 'name' table that can be used to refer to the axis in application
        user interfaces. The name ID must be greater than 255 and less than 32768.

        The postScriptNameID field provides a name ID that can be used to obtain
        strings from the 'name' table that can be treated as equivalent to name
        ID 6 (PostScript name) strings for the given instance. Values of 6 and
        "undefined" can be used; otherwise, values must be greater than 255 and
        less than 32768.

        The subfamilyNameID field provides a name ID that can be used to obtain
        strings from the 'name' table that can be treated as equivalent to name
        ID 17 (typographic subfamily) strings for the given instance. Values of
        2 or 17 can be used; otherwise, values must be greater than 255 and less
        than 32768.
    "#,
    proposal = "https://github.com/fonttools/fontbakery/issues/3703"
)]
fn valid_nameids(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.is_variable_font(), "not-variable", "Not a variable font");
    let mut problems = vec![];
    let valid_nameid = |n: NameId| (256..32768).contains(&n.to_u16());
    let valid_subfamily_nameid = |n: NameId| matches!(n.to_u16(), 2 | 17 | 256..32768);

    // Do the axes first
    for axis in f.font().axes().iter() {
        let axis_name_id = axis.name_id();
        if !valid_nameid(axis_name_id) {
            problems.push(Status::fail(
                &format!("invalid-axis-nameid:{}", axis_name_id.to_u16()),
                &format!(
                    "Axis name ID {} ({}) is out of range. It must be greater than 255 and less than 32768.",
                    axis_name_id.to_u16(), f.get_name_entry_strings(axis_name_id).next().unwrap_or_default()
                ),
            ));
        }
    }

    for instance in f.font().named_instances().iter() {
        let subfamily_name_id = instance.subfamily_name_id();
        if let Some(n) = instance.postscript_name_id() {
            if n != NameId::new(6) && !valid_nameid(n) {
                problems.push(Status::fail(
                        &format!("invalid-postscript-nameid:{}", n.to_u16()),
                        &format!(
                            "PostScript name ID {} ({}) is out of range. It must be greater than 255 and less than 32768, or 6 or 0xFFFF.",
                            n.to_u16(), f.get_name_entry_strings(n).next().unwrap_or_default()
                        ),
                    ));
            }
        }
        if !valid_subfamily_nameid(subfamily_name_id) {
            problems.push(Status::fail(
                &format!("invalid-subfamily-nameid:{}", subfamily_name_id.to_u16()),
                &format!(
                    "Instance subfamily name ID {} ({}) is out of range. It must be greater than 255 and less than 32768.",
                    subfamily_name_id.to_u16(), f.get_name_entry_strings(subfamily_name_id).next().unwrap_or_default()
                ),
            ));
        }
    }
    return_result(problems)
}
