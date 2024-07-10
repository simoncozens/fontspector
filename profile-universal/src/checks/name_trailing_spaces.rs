use fontspector_checkapi::{prelude::*, FileTypeConvert};
use read_fonts::TableProvider;

fn name_trailing_spaces(f: &Testable) -> StatusList {
    let mut problems: Vec<Status> = vec![];

    if let Ok(name_table) = TTF.from_testable(f).unwrap().font().name() {
        for name_record in name_table.name_record().iter() {
            if name_record
                .string(name_table.string_data())
                .map(|s| s.to_string())
                .map(|s| s.trim_end() != s)
                .unwrap_or(false)
            {
                problems.push(Status::fail(&format!(
                    "Name table record {:}/{:}/{:}/{:} has trailing spaces that must be removed:\n`{:}`",
                    name_record.platform_id,
                    name_record.encoding_id,
                    name_record.language_id,
                    name_record.name_id,
                    name_record.string(name_table.string_data()).unwrap(),
                )))
            }
        }
    }
    return_result(problems)
}

pub const NAME_TRAILING_SPACES_CHECK: Check = Check {
    id: "com.google.fonts/check/name/trailing_spaces",
    title: "Name table records must not have trailing spaces.",
    rationale: None,
    proposal: Some("https://github.com/googlefonts/fontbakery/issues/2417"),
    check_one: Some(&name_trailing_spaces),
    check_all: None,
    applies_to: "TTF",
};
