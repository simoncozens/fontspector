use fontspector_checkapi::{constants::RIBBI_STYLE_NAMES, prelude::*, testfont, FileTypeConvert};
use skrifa::string::StringId;

#[check(
    id = "googlefonts/name/mandatory_entries",
    rationale = "
        
        We require all fonts to have values for their font family name,
        font subfamily name, full font name, and postscript name. For RIBBI
        fonts, we also require values for the typographic family name and
        typographic subfamily name.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Font has all mandatory 'name' table entries?"
)]
fn mandatory_entries(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    let mut required_name_ids = vec![
        StringId::FAMILY_NAME,
        StringId::SUBFAMILY_NAME,
        StringId::FULL_NAME,
        StringId::POSTSCRIPT_NAME,
    ];
    if let Some(style) = f.style() {
        if !RIBBI_STYLE_NAMES.contains(&style) {
            required_name_ids.push(StringId::TYPOGRAPHIC_FAMILY_NAME);
            required_name_ids.push(StringId::TYPOGRAPHIC_SUBFAMILY_NAME);
        }
    }
    for name_id in required_name_ids {
        let strings = f.get_name_entry_strings(name_id).collect::<Vec<_>>();
        if strings.is_empty() || strings.iter().any(|s| s.is_empty()) {
            problems.push(Status::fail(
                "missing-entry",
                &format!(
                    "Font lacks entry with nameId={} ({:?})",
                    name_id.to_u16(),
                    name_id
                ),
            ));
        }
    }
    return_result(problems)
}
