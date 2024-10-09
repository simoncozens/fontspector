use std::collections::HashSet;

use fontspector_checkapi::constants::{VALID_FEATURE_TAGS, VALID_LANG_TAGS, VALID_SCRIPT_TAGS};
use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;

#[check(
    id = "opentype/layout_valid_language_tags",
    rationale = "
        Incorrect language tags can be indications of typos, leftover debugging code
        or questionable approaches, or user error in the font editor. Such typos can
        cause features and language support to fail to work as intended.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3355",
    title = "Does the font have any invalid language tags?"
)]
fn layout_valid_language_tags(f: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    let mut bad_tag = HashSet::new();

    if let Ok(gpos) = font.font().gpos() {
        let scriptlist = gpos.script_list()?;
        for script in scriptlist.script_records() {
            for lang in script.script(scriptlist.offset_data())?.lang_sys_records() {
                let tag = lang.lang_sys_tag().to_string();
                if !VALID_LANG_TAGS.contains(&tag.as_str()) {
                    bad_tag.insert(tag);
                }
            }
        }
    }

    if let Ok(gsub) = font.font().gsub() {
        let scriptlist = gsub.script_list()?;
        for script in scriptlist.script_records() {
            for lang in script.script(scriptlist.offset_data())?.lang_sys_records() {
                let tag = lang.lang_sys_tag().to_string();
                if !VALID_LANG_TAGS.contains(&tag.as_str()) {
                    bad_tag.insert(tag);
                }
            }
        }
    }

    Ok(if bad_tag.is_empty() {
        Status::just_one_pass()
    } else {
        Status::just_one_fail(
            "bad-language-tags",
            &format!(
                "The following invalid language tags were found in the font: {}",
                bad_tag.into_iter().collect::<Vec<_>>().join(", ")
            ),
        )
    })
}

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

    if let Ok(gpos) = font.font().gpos() {
        let scriptlist = gpos.script_list()?;
        for script in scriptlist.script_records() {
            let tag = script.script_tag().to_string();
            if !VALID_SCRIPT_TAGS.contains(&tag.as_str()) {
                bad_tag.insert(tag);
            }
        }
    }

    if let Ok(gsub) = font.font().gsub() {
        let scriptlist = gsub.script_list()?;
        for script in scriptlist.script_records() {
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
            "bad-language-tags",
            &format!(
                "The following invalid script tags were found in the font: {}",
                bad_tag.into_iter().collect::<Vec<_>>().join(", ")
            ),
        )
    })
}

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
    let mut bad_tag = HashSet::new();

    if let Ok(gpos) = font.font().gpos() {
        for feature_record in gpos.feature_list()?.feature_records() {
            let tag = feature_record.feature_tag().to_string();
            if !VALID_FEATURE_TAGS.contains(&tag.as_str())
                && (tag.len() != 4 || !tag.chars().all(|c| c.is_ascii_uppercase()))
            {
                bad_tag.insert(tag);
            }
        }
    }

    if let Ok(gsub) = font.font().gsub() {
        for feature_record in gsub.feature_list()?.feature_records() {
            let tag = feature_record.feature_tag().to_string();
            if !VALID_FEATURE_TAGS.contains(&tag.as_str())
                && (tag.len() != 4 || !tag.chars().all(|c| c.is_ascii_uppercase()))
            {
                bad_tag.insert(tag);
            }
        }
    }

    Ok(if bad_tag.is_empty() {
        Status::just_one_pass()
    } else {
        Status::just_one_fail(
            "bad-language-tags",
            &format!(
                "The following invalid feature tags were found in the font: {}",
                bad_tag.into_iter().collect::<Vec<_>>().join(", ")
            ),
        )
    })
}
