use fontspector_checkapi::prelude::*;
use itertools::Itertools;
use similar::{ChangeTag, TextDiff};

use crate::constants::OFL_BODY_TEXT;

#[check(
    id = "googlefonts/license/OFL_body_text",
    rationale = "
        Check OFL body text is correct.
        Often users will accidently delete parts of the body text.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3352",
    title = "Check OFL body text is correct.",
    applies_to = "LICENSE"
)]
fn OFL_body_text(t: &Testable, _context: &Context) -> CheckFnResult {
    let mut license_contents = String::from_utf8(t.contents.clone())
        .map_err(|e| CheckError::Error(format!("OFL.txt is not valid UTF-8: {:?}", e)))?;
    if license_contents.ends_with('\n') {
        license_contents.pop();
    }
    license_contents = license_contents
        .replace("http://", "https://")
        .replace("https://scripts.sil.org/OFL", "https://openfontlicense.org")
        .replace("<", "\\<")
        .split('\n')
        .map(|line| line.trim_end())
        .skip(1)
        .join("\n");
    if license_contents != OFL_BODY_TEXT {
        let diff = TextDiff::from_lines(OFL_BODY_TEXT, &license_contents);
        let mut changes: Vec<String> = vec![];
        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Delete => changes.push("-".to_string() + change.value()),
                ChangeTag::Insert => changes.push("+".to_string() + change.value()),
                ChangeTag::Equal => {}
            };
        }
        return Ok(Status::just_one_warn("incorrect-ofl-body-text", 
            &format!("The OFL.txt body text is incorrect. Please use https://github.com/googlefonts/Unified-Font-Repository/blob/main/OFL.txt as a template. You should only modify the first line.\n\nLines changed:\n\n{}",
            changes.join(""))));
    }
    return Ok(Status::just_one_pass());
}
