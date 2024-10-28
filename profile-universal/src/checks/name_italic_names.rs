use font_types::NameId;
use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};

#[check(
    id = "name/italic_names",
    rationale = "
        This check ensures that several entries in the name table
        conform to the font's Upright or Italic style,
        namely IDs 1 & 2 as well as 16 & 17 if they're present.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3666",
    title = "Check name table IDs 1, 2, 16, 17 to conform to Italic style."
)]
fn name_italic_names(t: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(t);
    let mut problems = vec![];
    let style = font.style();
    if let Some(style) = style {
        if !style.contains("Italic") {
            skip!("not-italic", "Font is not italic");
        }
    } else {
        skip!("not-italic", "Font is not italic");
    }
    if let Some(family_name) = font.get_name_entry_strings(NameId::FAMILY_NAME).next() {
        if family_name.contains("Italic") {
            problems.push(Status::fail(
                "bad-familyname",
                "Name ID 1 (Family Name) must not contain 'Italic'.",
            ));
        }
    }
    if let Some(subfamily_name) = font.get_name_entry_strings(NameId::SUBFAMILY_NAME).next() {
        if subfamily_name != "Italic" && subfamily_name != "Bold Italic" {
            problems.push(Status::fail(
                "bad-subfamilyname",
                &format!("Name ID 2 (Subfamily Name) does not conform to specs. Only R/I/B/BI are allowed, found {}", subfamily_name)
            ));
        }
    }
    if let Some(typo_family_name) = font
        .get_name_entry_strings(NameId::TYPOGRAPHIC_FAMILY_NAME)
        .next()
    {
        if typo_family_name.contains("Italic") {
            problems.push(Status::fail(
                "bad-typographicfamilyname",
                "Name ID 16 (Typographic Family Name) must not contain 'Italic'.",
            ));
        }
    }
    if let Some(typo_subfamily_name) = font
        .get_name_entry_strings(NameId::TYPOGRAPHIC_SUBFAMILY_NAME)
        .next()
    {
        if !typo_subfamily_name.ends_with("Italic") {
            problems.push(Status::fail(
                "bad-typographicsubfamilyname",
                "Name ID 16 (Typographic Family Name) must contain 'Italic'.",
            ));
        }
    }
    return_result(problems)
}
