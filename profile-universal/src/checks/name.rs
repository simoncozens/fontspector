use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;

fn name_empty_records(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let name = f.font().name()?;
    let mut problems: Vec<Status> = vec![];
    for record in name.name_record() {
        if record
            .string(name.string_data())?
            .to_string()
            .trim()
            .is_empty()
        {
            problems.push(Status::fail(
                "empty-record",
                &format!(
                    "Empty name record found for name ID={} platform ID={} encoding ID={}",
                    record.name_id(),
                    record.platform_id(),
                    record.encoding_id(),
                ),
            ));
        }
    }
    return_result(problems)
}

pub const CHECK_NAME_EMPTY_RECORDS: Check = Check {
    id: "com.google.fonts/check/name/empty_records",
    title: "Check name table for empty records.",
    rationale: "Check the name table for empty records, as this can cause problems in Adobe apps.",
    proposal: "https://github.com/fonttools/fontbakery/pull/2369",
    implementation: CheckImplementation::CheckOne(&name_empty_records),
    applies_to: "TTF",
    hotfix: None,
    fix_source: None,
    flags: CheckFlags::default(),
};
