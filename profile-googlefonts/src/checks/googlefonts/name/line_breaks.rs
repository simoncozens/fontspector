use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;

#[check(
    id = "googlefonts/name/line_breaks",
    rationale = "
        
        There are some entries on the name table that may include more than one line
        of text. The Google Fonts team, though, prefers to keep the name table entries
        short and simple without line breaks.

        For instance, some designers like to include the full text of a font license in
        the \"copyright notice\" entry, but for the GFonts collection this entry should
        only mention year, author and other basic info in a manner enforced by
        `googlefonts/font_copyright`
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Name table entries should not contain line-breaks."
)]
fn line_breaks(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    let name = f.font().name()?;
    for record in name.name_record().iter() {
        let string = record.string(name.string_data())?;
        if string.chars().any(|c| c == '\n') {
            problems.push(Status::fail(
                "line-break",
                &format!(
                    "Name entry {} on platform {} contains a line-break.",
                    record.name_id(),
                    record.platform_id()
                ),
            ));
        }
    }
    return_result(problems)
}
