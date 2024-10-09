use font_types::NameId;
use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;

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
    id = "opentype/name/no_copyright_on_description",
    rationale = "
        The name table in a font file contains strings about the font;
        there are entries for a copyright field and a description. If the
        copyright entry is being used correctly, then there should not
        be any copyright information in the description entry.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Description strings in the name table must not contain copyright info"
)]
fn check_name_no_copyright_on_description(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems: Vec<Status> = vec![];
    for record in f.get_name_entry_strings(NameId::DESCRIPTION) {
        if record.contains("opyright") {
            problems.push(Status::fail(
                "copyright-on-description",
                &format!(
                    "Some namerecords with  ID={} (NameID.DESCRIPTION) containing copyright info
should be removed (perhaps these were added by a longstanding FontLab Studio
5.x bug that copied copyright notices to them.)",
                    NameId::DESCRIPTION.to_u16()
                ),
            ))
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

        set_name_entry(&mut font,
                       3,  // PlatformID.WINDOWS
                       1,  // WindowsEncodingID.UNICODE_BMP
                       0x0409,  // WindowsLanguageID.ENGLISH_USA,
                       NameId::FAMILY_NAME,
                       "".to_string());

        assert_results_contain(
            run_check(super::name_empty_records, font),
            StatusCode::Fail, Some("empty-record".to_string()));
    }

    #[test]
    fn fail_with_a_string_that_only_has_whitespace() {
        let mut font: Testable = TEST_FILE!("source-sans-pro/TTF/SourceSansPro-Regular.ttf");

        set_name_entry(&mut font,
                       3,  // PlatformID.WINDOWS
                       1,  // WindowsEncodingID.UNICODE_BMP
                       0x0409,  // WindowsLanguageID.ENGLISH_USA,
                       NameId::FAMILY_NAME,
                       " ".to_string());

        assert_results_contain(
            run_check(super::name_empty_records, font),
            StatusCode::Fail, Some("empty-record".to_string()));
    }
}
