use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;

#[check(
    id = "no_mac_entries",
    rationale = "
        Mac name table entries are not needed anymore. Even Apple stopped producing
        name tables with platform 1. Please see for example the following system font:

        /System/Library/Fonts/SFCompact.ttf

        Also, Dave Opstad, who developed Apple's TrueType specifications, told
        Olli Meier a couple years ago (as of January/2022) that these entries are
        outdated and should not be produced anymore.",
    proposal = "https://github.com/googlefonts/gftools/issues/469",
    title = "Ensure font doesn't have Mac name table entries (platform=1)."
)]
fn no_mac_entries(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    for rec in f.font().name()?.name_record() {
        if rec.platform_id() == 1 {
            problems.push(Status::fail(
                "mac-names",
                &format!("Please remove name ID {}", rec.name_id()),
            ))
        }
    }
    return_result(problems)
}
