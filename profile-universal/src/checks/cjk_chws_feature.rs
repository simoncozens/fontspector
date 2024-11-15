use std::collections::HashSet;

use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use skrifa::Tag;

#[check(
    id = "cjk_chws_feature",
    rationale = "
        The W3C recommends the addition of chws and vchw features to CJK fonts
        to enhance the spacing of glyphs in environments which do not fully support
        JLREQ layout rules.

        The chws_tool utility (https://github.com/googlefonts/chws_tool) can be used
        to add these features automatically.
    ",
    title = "Does the font contain chws and vchw features?",
    proposal = "https://github.com/fonttools/fontbakery/issues/3363"
)]
fn cjk_chws_feature(f: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    let mut problems = vec![];
    skip!(!font.is_cjk_font(), "not-cjk", "Not a CJK font.");
    let message = "feature not found in font. Use chws_tool (https://github.com/googlefonts/chws_tool) to add it.";
    let tags: HashSet<Tag> = font
        .feature_records(false)
        .map(|(r, _)| r.feature_tag())
        .collect();
    if !tags.contains(&Tag::new(b"chws")) {
        problems.push(Status::warn(
            "missing-chws-feature",
            &format!("chws {}", message),
        ));
    }
    if !tags.contains(&Tag::new(b"vchw")) {
        problems.push(Status::warn(
            "missing-vchw-feature",
            &format!("vchw {}", message),
        ));
    }
    return_result(problems)
}
