use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use skrifa::MetadataProvider;

const REGISTERED_AXIS_TAGS: [&str; 5] = ["ital", "opsz", "slnt", "wdth", "wght"];

#[check(
    id = "opentype/varfont/foundry_defined_tag_name",
    title = "Validate foundry-defined design-variation axis tag names.",
    rationale = "According to the OpenType spec's syntactic requirements for
    foundry-defined design-variation axis tags available at
    https://learn.microsoft.com/en-us/typography/opentype/spec/dvaraxisreg

    Foundry-defined tags must begin with an uppercase letter
    and must use only uppercase letters or digits.",
    proposal = "https://github.com/fonttools/fontbakery/issues/4043"
)]
fn foundry_defined_tag_name(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.is_variable_font(), "not-variable", "Not a variable font");
    let mut problems = vec![];
    for axis in f.font().axes().iter() {
        let tag = axis.tag().to_string();
        if REGISTERED_AXIS_TAGS.contains(&tag.as_str()) {
            continue;
        }
        if REGISTERED_AXIS_TAGS.contains(&tag.to_lowercase().as_str()) {
            problems.push(Status::warn("foundry-defined-similar-registered-name",
                &format!("Foundry-defined axis tag {} is similar to a registered tag name {}, consider renaming. If this tag was meant to be a registered tag, please use all lowercase letters in the tag name.", tag, tag.to_lowercase())
            ));
        }
        // Axis tag must be uppercase and contain only uppercase letters or digits
        if !tag
            .chars()
            .next()
            .map(|c| c.is_ascii_uppercase())
            .unwrap_or(false)
        {
            problems.push(Status::fail(
                "invalid-foundry-defined-tag-first-letter",
                &format!(
                    "Foundry-defined axis tag {} must begin with an uppercase letter",
                    tag
                ),
            ))
        } else if !tag
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        {
            problems.push(Status::fail("invalid-foundry-defined-tag-chars",
                &format!("Foundry-defined axis tag {} must begin with an uppercase letter and contain only uppercase letters or digits.", tag)
            ));
        }
    }
    return_result(problems)
}
