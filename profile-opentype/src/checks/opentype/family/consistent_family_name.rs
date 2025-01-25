use std::collections::HashMap;

use fontspector_checkapi::{prelude::*, FileTypeConvert};
use read_fonts::types::NameId;
use skrifa::MetadataProvider;


#[check(
    id = "opentype/family/consistent_family_name",
    rationale = r#"
        Per the OpenType spec:

            * "...many existing applications that use this pair of names assume that a
              Font Family name is shared by at most four fonts that form a font
              style-linking group"

            * "For extended typographic families that includes fonts other than the
              four basic styles(regular, italic, bold, bold italic), it is strongly
              recommended that name IDs 16 and 17 be used in fonts to create an
              extended, typographic grouping."

            * "If name ID 16 is absent, then name ID 1 is considered to be the
              typographic family name."

        https://learn.microsoft.com/en-us/typography/opentype/spec/name

        Fonts within a font family all must have consistent names
        in the Typographic Family name (nameID 16)
        or Font Family name (nameID 1), depending on which it uses.

        Inconsistent font/typographic family names across fonts in a family
        can result in unexpected behaviors, such as broken style linking.
    "#,
    proposal = "https://github.com/fonttools/fontbakery/issues/4112",
    title = "Verify that family names in the name table are consistent across all fonts in the family. Checks Typographic Family name (nameID 16) if present, otherwise uses Font Family name (nameID 1)",
    implementation = "all"
)]
fn consistent_family_name(c: &TestableCollection, context: &Context) -> CheckFnResult {
    let fonts = TTF.from_collection(c);
    let mut problems = vec![];
    let mut family_names = HashMap::new();
    for font in fonts {
        let family_name = font
            .font()
            .localized_strings(NameId::TYPOGRAPHIC_FAMILY_NAME)
            .english_or_first()
            .or_else(|| {
                font.font()
                    .localized_strings(NameId::FAMILY_NAME)
                    .english_or_first()
            })
            .ok_or_else(|| {
                CheckError::Error(format!(
                    "Font {} is missing a Family Name entry",
                    font.filename.to_string_lossy()
                ))
            })?
            .chars()
            .collect::<String>();
        family_names
            .entry(family_name)
            .or_insert_with(Vec::new)
            .push(font.filename.to_string_lossy().to_string());
    }
    if family_names.len() > 1 {
        let report = bullet_list(
            context,
            family_names
                .iter()
                .map(|(name, fonts)| format!("'{}' (found in fonts {})", name, fonts.join(", "))),
        );
        problems.push(Status::fail(
            "inconsistent-family-name",
            &format!(
                "{} different family names were found:\n\n{}",
                family_names.len(),
                report
            ),
        ));
    }
    return_result(problems)
}
