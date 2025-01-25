use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::types::NameId;

#[check(
    id = "opentype/postscript_name",
    title = "PostScript name follows OpenType specification requirements?",
    rationale = "The PostScript name is used by some applications to identify the font. It should only consist of characters from the set A-Z, a-z, 0-9, and hyphen.",
    proposal = "https://github.com/miguelsousa/openbakery/issues/62"
)]
fn postscript_name(t: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(t);
    let mut problems = vec![];
    for name in font.get_name_entry_strings(NameId::POSTSCRIPT_NAME) {
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            problems.push(Status::fail(
                "bad-psname-entries",
                &format!("PostScript name '{}' contains invalid characters", name),
            ));
        }
    }
    return_result(problems)
}
