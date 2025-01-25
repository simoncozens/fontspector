use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::types::NameId;

const NAME_LIMITS: [(NameId, usize); 6] = [
    (NameId::FULL_NAME, 63),
    (NameId::POSTSCRIPT_NAME, 63),
    (NameId::FAMILY_NAME, 31),
    (NameId::SUBFAMILY_NAME, 31),
    (NameId::TYPOGRAPHIC_FAMILY_NAME, 31),
    (NameId::TYPOGRAPHIC_SUBFAMILY_NAME, 31),
];

#[check(
    id = "opentype/family_naming_recommendations",
    rationale = "
        This check ensures that the length of various family name and style
        name strings in the name table are within the maximum length
        recommended by the OpenType specification.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Font follows the family naming recommendations?"
)]
fn family_naming_recommendations(t: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(t);

    let mut problems = vec![];
    for (name_id, max_length) in NAME_LIMITS.iter() {
        for name in font.get_name_entry_strings(*name_id) {
            if name.len() > *max_length {
                problems.push(Status::info(
                    "bad-entries",
                    &format!(
                        "{:?} (\"{}\") is too long ({} > {})",
                        name_id,
                        name,
                        name.len(),
                        max_length
                    ),
                ));
            }
        }
    }
    return_result(problems)
}
