use itertools::Itertools;
use std::collections::HashSet;

use fontspector_checkapi::{prelude::*, FileTypeConvert};

fn family_equal_codepoint_coverage(c: &Testable, _context: &Context) -> CheckFnResult {
    let f = TTF.from_testable(c).ok_or("Not a TTF file")?;
    let siblings = f.siblings();
    if siblings.is_empty() {
        return return_result(vec![Status::skip("No sibling fonts found")]);
    }
    let mut problems = vec![];
    let mut we_have_they_dont: HashSet<u32> = HashSet::new();
    let mut they_have_we_dont: HashSet<u32> = HashSet::new();
    let my_codepoints = f.codepoints();
    for sibling in siblings {
        let their_codepoints = sibling.codepoints();
        we_have_they_dont.extend(my_codepoints.difference(their_codepoints));
        they_have_we_dont.extend(their_codepoints.difference(my_codepoints));
    }

    if !we_have_they_dont.is_empty() {
        problems.push(Status::fail(&format!(
            "Our font has codepoints not present in sibling fonts: {}",
            we_have_they_dont
                .iter()
                .map(|i| format!("U+{:04X}", i))
                .join(", ")
        )))
    }
    if !they_have_we_dont.is_empty() {
        problems.push(Status::fail(&format!(
            "Other fonts have codepoints not present in this font: {}",
            they_have_we_dont
                .iter()
                .map(|i| format!("U+{:04X}", i))
                .join(", ")
        )))
    }
    return_result(problems)
}

pub const CHECK_FAMILY_EQUAL_CODEPOINT_COVERAGE: Check = Check {
    id: "com.google.fonts/check/family/equal_codepoint_coverage",
    title: "Fonts have equal codepoint coverage?",
    rationale: "For a given family, all fonts must have the same codepoint coverage.
                This is because we want to avoid the situation where, for example,
                a character is present in a regular font but missing in the italic
                style; turning on italic would cause the character to be rendered
                either as a fake italic (auto-slanted) or to show tofu.",
    proposal: "https://github.com/fonttools/fontbakery/issues/4180",
    check_one: Some(&family_equal_codepoint_coverage),
    check_all: None,
    applies_to: "TTF",
    hotfix: None,
    fix_source: None,
};
