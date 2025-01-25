use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use read_fonts::types::{F2Dot14, NameId};
use skrifa::MetadataProvider;

#[check(
    id = "opentype/varfont/valid_default_instance_nameids",
    title = "Validates subfamilyNameID and postScriptNameID for the default instance record",
    rationale = r#"
        According to the 'fvar' documentation in OpenType spec v1.9.1
        https://docs.microsoft.com/en-us/typography/opentype/spec/fvar

        The default instance of a font is that instance for which the coordinate
        value of each axis is the defaultValue specified in the corresponding
        variation axis record. An instance record is not required for the default
        instance, though an instance record can be provided. When enumerating named
        instances, the default instance should be enumerated even if there is no
        corresponding instance record. If an instance record is included for the
        default instance (that is, an instance record has coordinates set to default
        values), then the nameID value should be set to either 2 or 17 or to a
        name ID with the same value as name ID 2 or 17. Also, if a postScriptNameID is
        included in instance records, and the postScriptNameID value should be set
        to 6 or to a name ID with the same value as name ID 6.
    "#,
    proposal = "https://github.com/fonttools/fontbakery/issues/3708"
)]
fn valid_default_instance_nameids(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.is_variable_font(), "not-variable", "Not a variable font");
    let mut problems = vec![];
    let has_a_postscriptname = f
        .font()
        .named_instances()
        .iter()
        .any(|ni| ni.postscript_name_id().is_some());
    let name2 = f
        .get_name_entry_strings(NameId::new(2))
        .next()
        .unwrap_or_default();
    let name6 = f
        .get_name_entry_strings(NameId::new(6))
        .next()
        .unwrap_or_default();
    let name17 = f
        .get_name_entry_strings(NameId::new(17))
        .next()
        .unwrap_or_default();
    let font_subfamily_name = if !name17.is_empty() {
        name17.clone()
    } else {
        name2.clone()
    };
    let default_coords = vec![F2Dot14::from_f32(0.0); f.font().axes().len()];
    for (index, instance) in f.font().named_instances().iter().enumerate() {
        if instance.location().coords() != default_coords {
            continue;
        }
        let subfamily_name = f
            .get_name_entry_strings(instance.subfamily_name_id())
            .next()
            .unwrap_or_else(|| format!("instance {}", index + 1));
        let postscript_name = instance
            .postscript_name_id()
            .and_then(|n| f.get_name_entry_strings(n).next())
            .unwrap_or("None".to_string());
        if !name17.is_empty() && subfamily_name != font_subfamily_name {
            problems.push(Status::fail("invalid-default-instance-subfamily-name", &format!(
                "{} instance has the same coordinates as the default instance; its subfamily name should be {}.\n\nNote: It is alternatively possible that Name ID 17 is incorrect, and should be set to the default instance subfamily name, {}, rather than '{}'. If the default instance is {}, NameID 17 is probably the problem.",
                subfamily_name, font_subfamily_name, font_subfamily_name, name17, subfamily_name
            )))
        }
        if name17.is_empty() && subfamily_name != font_subfamily_name {
            problems.push(Status::fail("invalid-default-instance-subfamily-name", &format!(
                "{} instance has the same coordinates as the default instance; its subfamily name should be {}.\n\nNote: If the default instance really is meant to be called {}, the problem may be that the font lacks NameID 17, which should probably be present and set to {}.",
                subfamily_name, font_subfamily_name, subfamily_name, subfamily_name
            )))
        }
        if has_a_postscriptname && postscript_name != name6 {
            problems.push(Status::fail("invalid-default-instance-postscript-name", &format!(
                "{} instance has the same coordinates as the default instance; its postscript name should be {} instead of {}.",
                subfamily_name, name6, postscript_name
            )));
        }
    }
    return_result(problems)
}
