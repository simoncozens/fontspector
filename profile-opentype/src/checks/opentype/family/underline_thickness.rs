use std::path::PathBuf;

use fontspector_checkapi::{prelude::*, FileTypeConvert};
use itertools::Itertools;
use read_fonts::{types::FWord, TableProvider};

#[check(
    id = "opentype/family/underline_thickness",
    title = "Fonts have consistent underline thickness?",
    rationale = r#"
        Dave C Lemon (Adobe Type Team) recommends setting the underline thickness to be
        consistent across the family.

        If thicknesses are not family consistent, words set on the same line which have
        different styles look strange.
    "#,
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",  // legacy check
    implementation = "all"
)]
fn underline_thickness(c: &TestableCollection, _context: &Context) -> CheckFnResult {
    let fonts = TTF.from_collection(c);
    if fonts.len() < 2 {
        return Err(CheckError::Skip {
            code: "no-siblings".to_string(),
            message: "No sibling fonts found".to_string(),
        });
    }
    let posts: Vec<(&PathBuf, FWord)> = fonts
        .iter()
        .map(|font| {
            (
                &font.filename,
                font.font()
                    .post()
                    .map(|post| post.underline_thickness())
                    .unwrap_or_default(),
            )
        })
        .collect();
    Ok(if posts.iter().unique_by(|(_, t)| t).count() == 1 {
        Status::just_one_pass()
    } else {
        let mut message =
            "Underline thickness is inconsistent. Detected underline thickness values are:\n\n"
                .to_string();
        for (path, thickness) in posts {
            message.push_str(&format!("* {}: {}\n", path.display(), thickness));
        }
        Status::just_one_fail("inconsistent-underline-thickness", &message)
    })
}
