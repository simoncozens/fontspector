use std::collections::HashSet;

use fontspector_checkapi::{prelude::*, FileTypeConvert};
use read_fonts::tables::os2::SelectionFlags;

#[check(
    id = "opentype/family/bold_italic_unique_for_nameid1",
    title = "Check that OS/2.fsSelection bold & italic settings are unique for each NameID1",
    rationale = "Per the OpenType spec: name ID 1 'is used in combination with Font Subfamily
        name (name ID 2), and should be shared among at most four fonts that differ
        only in weight or style.

        This four-way distinction should also be reflected in the OS/2.fsSelection
        field, using bits 0 and 5.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",  // legacy check
    implementation = "all"
)]
fn bold_italic_unique(c: &TestableCollection, _context: &Context) -> CheckFnResult {
    let fonts = TTF.from_collection(c);
    let mut problems = vec![];
    let mut flags: HashSet<(bool, bool)> = HashSet::new();
    let ribbi = fonts.iter().filter(|f| f.is_ribbi());
    for font in ribbi {
        let fsselection = font.get_os2_fsselection()?;
        let val = (
            fsselection.intersects(SelectionFlags::BOLD),
            fsselection.intersects(SelectionFlags::ITALIC),
        );
        if flags.contains(&val) {
            problems.push(Status::fail(
                "unique-fsselection",
                &(format!(
                    "Font {} has the same selection flags ({}{}{}) as another font",
                    font.filename.to_string_lossy(),
                    if val.0 { "bold" } else { "" },
                    if val.0 && val.1 { " & " } else { "" },
                    if val.1 { "italic" } else { "" }
                )),
            ));
        } else {
            flags.insert(val);
        }
    }
    return_result(problems)
}
