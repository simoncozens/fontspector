use std::collections::HashMap;

use font_types::NameId;
use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use read_fonts::TableProvider;
use skrifa::MetadataProvider;

#[check(
    id = "opentype/name/empty_records",
    title = "Check name table for empty records.",
    rationale = "Check the name table for empty records, as this can cause problems in Adobe apps.",
    proposal = "https://github.com/fonttools/fontbakery/pull/2369"
)]
fn name_empty_records(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let name = f.font().name()?;
    let mut problems: Vec<Status> = vec![];
    for record in name.name_record() {
        if record
            .string(name.string_data())?
            .to_string()
            .trim()
            .is_empty()
        {
            problems.push(Status::fail(
                "empty-record",
                &format!(
                    "Empty name record found for name ID={} platform ID={} encoding ID={}",
                    record.name_id(),
                    record.platform_id(),
                    record.encoding_id(),
                ),
            ));
        }
    }
    return_result(problems)
}

#[check(
    id = "opentype/name/match_familyname_fullfont",
    rationale = r#"
        The FULL_FONT_NAME entry in the ‘name’ table should start with the same string
        as the Family Name (FONT_FAMILY_NAME, TYPOGRAPHIC_FAMILY_NAME or
        WWS_FAMILY_NAME).

        If the Family Name is not included as the first part of the Full Font Name, and
        the user embeds the font in a document using a Microsoft Office app, the app
        will fail to render the font when it opens the document again.

        NOTE: Up until version 1.5, the OpenType spec included the following exception
        in the definition of Full Font Name:

            "An exception to the [above] definition of Full font name is for Microsoft
            platform strings for CFF OpenType fonts: in this case, the Full font name
            string must be identical to the PostScript FontName in the CFF Name INDEX."

        https://docs.microsoft.com/en-us/typography/opentype/otspec150/name#name-ids
    "#,
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Does full font name begin with the font family name?"
)]
fn check_name_match_familyname_fullfont(t: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(t);
    // We actually care about localization here, so don't just want
    // a vec of String.
    let full_names = font.get_name_entry_strings(NameId::FULL_NAME);
    let family_names = font
        .get_name_entry_strings(NameId::FAMILY_NAME)
        .collect::<Vec<_>>();
    let typographic_names = font
        .get_name_entry_strings(NameId::TYPOGRAPHIC_FAMILY_NAME)
        .collect::<Vec<_>>();
    let wws_names = font
        .get_name_entry_strings(NameId::WWS_FAMILY_NAME)
        .collect::<Vec<_>>();
    for name in full_names {
        if !family_names.iter().any(|f| name.starts_with(f))
            && !typographic_names.iter().any(|f| name.starts_with(f))
            && !wws_names.iter().any(|f| name.starts_with(f))
        {
            return return_result(vec![Status::fail(
                "mismatch-font-names",
                &format!(
                    "Full font name '{}' does not start with the Family Name",
                    name
                ),
            )]);
        }
    }
    return_result(vec![])
}

#[check(
    id = "opentype/family/max_4_fonts_per_family_name",
    rationale = "
        Per the OpenType spec:

        'The Font Family name [...] should be shared among at most four fonts that
        differ only in weight or style [...]'
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/2372",
    title = "Verify that each group of fonts with the same nameID 1 has maximum of 4 fonts.",
    implementation = "all"
)]
fn family_max_4_fonts_per_family_name(t: &TestableCollection, _context: &Context) -> CheckFnResult {
    let fonts = TTF.from_collection(t);
    let mut counter = HashMap::new();
    let mut problems = vec![];
    for font in fonts {
        let family_name = font
            .get_name_entry_strings(NameId::FAMILY_NAME)
            .next()
            .ok_or_else(|| {
                CheckError::Error(format!(
                    "Font {} is missing a Family Name entry",
                    font.filename.to_string_lossy()
                ))
            })?;
        *counter.entry(family_name).or_insert(0) += 1;
    }
    for (family_name, count) in counter {
        if count > 4 {
            problems.push(Status::fail(
                "too-many-fonts",
                &format!(
                    "Family name '{}' has {} fonts, which is more than the maximum of 4",
                    family_name, count
                ),
            ));
        }
    }

    return_result(problems)
}

#[check(
    id = "opentype/postscript_name",
    title = "PostScript name follows OpenType specification requirements?",
    rationale = "The PostScript name is used by some applications to identify the font. It should only consist of characters from the set A-Z, a-z, 0-9, and hyphen.",
    proposal = "https://github.com/miguelsousa/openbakery/issues/62"
)]
fn postscript_name(t: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(t);
    let mut problems = vec![];
    for name in font.get_name_entry_strings(NameId::POSTSCRIPT_NAME) {
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            problems.push(Status::fail(
                "invalid-postscript-name",
                &format!("PostScript name '{}' contains invalid characters", name),
            ));
        }
    }
    return_result(problems)
}

const NAME_LIMITS: [(NameId, usize); 6] = [
    (NameId::FULL_NAME, 63),
    (NameId::POSTSCRIPT_NAME, 63),
    (NameId::FAMILY_NAME, 31),
    (NameId::SUBFAMILY_NAME, 31),
    (NameId::TYPOGRAPHIC_FAMILY_NAME, 31),
    (NameId::TYPOGRAPHIC_SUBFAMILY_NAME, 31),
];

#[check(
    id = "opentype/family_naming_recommendations",
    rationale = "
        This check ensures that the length of various family name and style
        name strings in the name table are within the maximum length
        recommended by the OpenType specification.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Font follows the family naming recommendations?"
)]
fn family_naming_recommendations(t: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(t);

    let mut problems = vec![];
    for (name_id, max_length) in NAME_LIMITS.iter() {
        for name in font.get_name_entry_strings(*name_id) {
            if name.len() > *max_length {
                problems.push(Status::warn(
                    "name-too-long",
                    &format!(
                        "{:?} (\"{}\") is too long ({} > {})",
                        name_id,
                        name,
                        name.len(),
                        max_length
                    ),
                ));
            }
        }
    }
    return_result(problems)
}

#[check(
    id = "opentype/name/italic_names",
    rationale = "
        This check ensures that several entries in the name table
        conform to the font's Upright or Italic style,
        namely IDs 1 & 2 as well as 16 & 17 if they're present.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3666",
    title = "Check name table IDs 1, 2, 16, 17 to conform to Italic style."
)]
fn name_italic_names(t: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(t);
    let mut problems = vec![];
    skip!(
        !font.filename_suggests_italic(),
        "not-italic",
        "Font is not italic"
    );
    if let Some(family_name) = font.get_name_entry_strings(NameId::FAMILY_NAME).next() {
        if family_name.contains("Italic") {
            problems.push(Status::fail(
                "bad-familyname",
                "Name ID 1 (Family Name) must not contain 'Italic'.",
            ));
        }
    }
    if let Some(subfamily_name) = font.get_name_entry_strings(NameId::SUBFAMILY_NAME).next() {
        if subfamily_name != "Italic" && subfamily_name != "Bold Italic" {
            problems.push(Status::fail(
                "bad-subfamilyname",
                &format!("Name ID 2 (Subfamily Name) does not conform to specs. Only R/I/B/BI are allowed, found {}", subfamily_name)
            ));
        }
    }
    if let Some(typo_family_name) = font
        .get_name_entry_strings(NameId::TYPOGRAPHIC_FAMILY_NAME)
        .next()
    {
        if typo_family_name.contains("Italic") {
            problems.push(Status::fail(
                "bad-typographicfamilyname",
                "Name ID 16 (Typographic Family Name) must not contain 'Italic'.",
            ));
        }
    }
    if let Some(typo_subfamily_name) = font
        .get_name_entry_strings(NameId::TYPOGRAPHIC_SUBFAMILY_NAME)
        .next()
    {
        if !typo_subfamily_name.ends_with("Italic") {
            problems.push(Status::fail(
                "bad-typographicsubfamilyname",
                "Name ID 16 (Typographic Family Name) must contain 'Italic'.",
            ));
        }
    }
    return_result(problems)
}

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
                .map(|(name, fonts)| format!("{} (found in fonts {})", name, fonts.join(", "))),
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use fontspector_checkapi::{
        codetesting::{assert_pass, assert_results_contain, run_check, set_name_entry},
        StatusCode, TEST_FILE,
    };

    #[test]
    fn pass_opentype_name_empty_records_with_fully_populated_name_records() {
        let font: Testable = TEST_FILE!("source-sans-pro/TTF/SourceSansPro-Regular.ttf");
        assert_pass(run_check(super::name_empty_records, font));
    }

    #[test]
    fn fail_with_a_completely_empty_string() {
        let mut font: Testable = TEST_FILE!("source-sans-pro/TTF/SourceSansPro-Regular.ttf");

        set_name_entry(
            &mut font,
            3,      // PlatformID.WINDOWS
            1,      // WindowsEncodingID.UNICODE_BMP
            0x0409, // WindowsLanguageID.ENGLISH_USA,
            NameId::FAMILY_NAME,
            "".to_string(),
        );

        assert_results_contain(
            run_check(super::name_empty_records, font),
            StatusCode::Fail,
            Some("empty-record".to_string()),
        );
    }

    #[test]
    fn fail_with_a_string_that_only_has_whitespace() {
        let mut font: Testable = TEST_FILE!("source-sans-pro/TTF/SourceSansPro-Regular.ttf");

        set_name_entry(
            &mut font,
            3,      // PlatformID.WINDOWS
            1,      // WindowsEncodingID.UNICODE_BMP
            0x0409, // WindowsLanguageID.ENGLISH_USA,
            NameId::FAMILY_NAME,
            " ".to_string(),
        );

        assert_results_contain(
            run_check(super::name_empty_records, font),
            StatusCode::Fail,
            Some("empty-record".to_string()),
        );
    }
}
