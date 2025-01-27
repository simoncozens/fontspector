use std::collections::HashSet;

use fontspector_checkapi::{constants::VALID_FEATURE_TAGS, prelude::*, testfont, FileTypeConvert};

#[check(
    id = "opentype/layout_valid_feature_tags",
    rationale = "
        Incorrect tags can be indications of typos, leftover debugging code
        or questionable approaches, or user error in the font editor. Such typos can
        cause features and language support to fail to work as intended.

        Font vendors may use private tags to identify private features. These tags
        must be four uppercase letters (A-Z) with no punctuation, spaces, or numbers.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3355",
    title = "Does the font have any invalid feature tags?"
)]
fn layout_valid_feature_tags(f: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    let mut bad_tag: HashSet<_> = HashSet::new();
    for (feature_record, _) in font.feature_records(false) {
        let tag = feature_record.feature_tag().to_string();
        // ssXX and cvXX are OK.
        if (tag.starts_with("ss") || tag.starts_with("cv"))
            && tag[2..].chars().all(|c| c.is_ascii_digit())
        {
            continue;
        }
        if !VALID_FEATURE_TAGS.contains(&tag.as_str())
            && (tag.len() != 4 || !tag.chars().all(|c| c.is_ascii_uppercase()))
        {
            bad_tag.insert(tag);
        }
    }

    Ok(if bad_tag.is_empty() {
        Status::just_one_pass()
    } else {
        Status::just_one_fail(
            "bad-feature-tags",
            &format!(
                "The following invalid feature tags were found in the font: {}",
                bad_tag.into_iter().collect::<Vec<_>>().join(", ")
            ),
        )
    })
}
