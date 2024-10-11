use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;

#[check(
    id = "name/trailing_spaces",
    title = "Name table records must not have trailing spaces.",
    rationale = "This check ensures that no entries in the name table end in spaces;
                trailing spaces, particularly in font names, can be confusing to users.
                In most cases this can be fixed by removing trailing spaces from the
                metadata fields in the font editor.",
    proposal = "https://github.com/googlefonts/fontbakery/issues/2417",
    hotfix = fix_trailing_spaces
)]
fn name_trailing_spaces(f: &Testable, _context: &Context) -> CheckFnResult {
    let mut problems: Vec<Status> = vec![];

    if let Ok(name_table) = testfont!(f).font().name() {
        for name_record in name_table.name_record().iter() {
            if name_record
                .string(name_table.string_data())
                .map(|s| s.to_string())
                .map(|s| s.trim_end() != s)
                .unwrap_or(false)
            {
                problems.push(Status::fail("trailing-space",&format!(
                    "Name table record {:}/{:}/{:}/{:} has trailing spaces that must be removed:\n`{:}`",
                    name_record.platform_id,
                    name_record.encoding_id,
                    name_record.language_id,
                    name_record.name_id,
                    name_record.string(name_table.string_data()).map_err(|_| CheckError::Error("Error reading string".to_string()))?,
                )))
            }
        }
    }
    return_result(problems)
}

fn fix_trailing_spaces(_f: &Testable) -> FixFnResult {
    Ok(false)
}
