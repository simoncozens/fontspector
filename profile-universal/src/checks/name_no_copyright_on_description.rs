use font_types::NameId;
use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};

#[check(
    id = "name/no_copyright_on_description",
    rationale = "
        The name table in a font file contains strings about the font;
        there are entries for a copyright field and a description. If the
        copyright entry is being used correctly, then there should not
        be any copyright information in the description entry.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Description strings in the name table must not contain copyright info"
)]
fn name_no_copyright_on_description(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems: Vec<Status> = vec![];
    for record in f.get_name_entry_strings(NameId::DESCRIPTION) {
        if record.contains("opyright") {
            problems.push(Status::fail(
                "copyright-on-description",
                &format!(
                    "Some namerecords with  ID={} (NameID.DESCRIPTION) containing copyright info
should be removed (perhaps these were added by a longstanding FontLab Studio
5.x bug that copied copyright notices to them.)",
                    NameId::DESCRIPTION.to_u16()
                ),
            ))
        }
    }
    return_result(problems)
}

