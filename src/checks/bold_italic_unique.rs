use std::collections::HashSet;

use crate::{
    check::{return_result, Status, StatusCode, StatusList},
    Check, FontCollection,
};
use read_fonts::tables::os2::SelectionFlags;
use skrifa::string::StringId;

fn bold_italic_unique(c: &FontCollection) -> StatusList {
    let ribbi = c.ribbi_fonts();
    let mut problems = vec![];
    let mut flags: HashSet<(bool, bool)> = HashSet::new();
    for font in ribbi.iter() {
        let _names_list = font.get_name_entry_strings(StringId::FAMILY_NAME);
        match font.get_os2_fsselection() {
            Ok(fsselection) => {
                let val = (
                    fsselection.intersects(SelectionFlags::BOLD),
                    fsselection.intersects(SelectionFlags::ITALIC),
                );
                if flags.contains(&val) {
                    problems.push(Status {
                        message: Some(format!(
                            "Font {} has the same selection flags ({}{}{}) as another font",
                            font.filename,
                            if val.0 { "bold" } else { "" },
                            if val.0 && val.1 { " & " } else { "" },
                            if val.1 { "italic" } else { "" }
                        )),
                        code: StatusCode::Error,
                    });
                } else {
                    flags.insert(val);
                }
            }
            Err(_e) => problems.push(Status {
                message: Some(format!("Font {} had no OS2 table", font.filename)),
                code: StatusCode::Error,
            }),
        }
    }
    return_result(problems)
}
pub const BOLD_ITALIC_UNIQUE_CHECK: Check = Check {
    id: "com.adobe.fonts/check/family/bold_italic_unique_for_nameid1",
    title: "Check that OS/2.fsSelection bold & italic settings are unique for each NameID1",
    rationale: None,
    proposal: Some("https://github.com/googlefonts/fontbakery/pull/2388"),
    check_all: Some(&bold_italic_unique),
    check_one: None,
};
