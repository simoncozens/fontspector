use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::types::NameId;

#[check(
    id = "googlefonts/name/familyname_first_char",
    rationale = "
        
        Font family names which start with a numeral are often not discoverable
        in Windows applications.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Make sure family name does not begin with a digit."
)]
fn familyname_first_char(t: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(t);
    let mut problems = vec![];
    for family_name in font.get_name_entry_strings(NameId::FAMILY_NAME) {
        if "0123456789".chars().any(|c| family_name.starts_with(c)) {
            problems.push(Status::fail(
                "begins-with-digit",
                &format!("Font family name '{}' begins with a digit!", family_name),
            ));
        }
    }
    return_result(problems)
}
