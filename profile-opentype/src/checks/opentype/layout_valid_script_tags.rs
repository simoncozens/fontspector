use std::collections::HashSet;

use fontspector_checkapi::{
    constants::VALID_SCRIPT_TAGS,
    FileTypeConvert,
    prelude::*,
    testfont,
};
use read_fonts::TableProvider;

#[check(
    id = "opentype/layout_valid_script_tags",
    rationale = "
        Incorrect script tags can be indications of typos, leftover debugging code
        or questionable approaches, or user error in the font editor. Such typos can
        cause features and language support to fail to work as intended.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3355",
    title = "Does the font have any invalid script tags?"
)]
fn layout_valid_script_tags(f: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    let mut bad_tag = HashSet::new();

    let gsub_script_list = font
        .font()
        .gsub()
        .ok()
        .and_then(|gsub| gsub.script_list().ok());
    let gpos_script_list = font
        .font()
        .gsub()
        .ok()
        .and_then(|gsub| gsub.script_list().ok());
    for script_list in [gsub_script_list, gpos_script_list].iter().flatten() {
        for script in script_list.script_records() {
            let tag = script.script_tag().to_string();
            if !VALID_SCRIPT_TAGS.contains(&tag.as_str()) {
                bad_tag.insert(tag);
            }
        }
    }

    Ok(if bad_tag.is_empty() {
        Status::just_one_pass()
    } else {
        Status::just_one_fail(
            "bad-script-tags",
            &format!(
                "The following invalid script tags were found in the font: {}",
                bad_tag.into_iter().collect::<Vec<_>>().join(", ")
            ),
        )
    })
}
