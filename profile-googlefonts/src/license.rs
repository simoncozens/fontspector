use font_types::NameId;
use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;
use regex::Regex;
use skrifa::MetadataProvider;

// Although this is a /name/ check, it's really about licensing
#[check(
    id = "googlefonts/name/rfn",
    title = "Name table strings must not contain the string 'Reserved Font Name'.",
    rationale = "
        Some designers adopt the \"Reserved Font Name\" clause of the OFL license. This
        means that the original author reserves the rights to the family name and other
        people can only distribute modified versions using a different family name.

        Google Fonts published updates to the fonts in the collection in order to fix
        issues and/or implement further improvements to the fonts. It is important to
        keep the family name so that users of the webfonts can benefit from the updates.
        Since it would forbid such usage scenario, all families in the GFonts collection
        are required to not adopt the RFN clause.

        This check ensures \"Reserved Font Name\" is not mentioned in the name table.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/1380"
)]
fn name_rfn(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];

    let records = f.font().name()?.name_record();
    let str_data = f.font().name()?.string_data();

    for name in records {
        let name_string = name
            .string(str_data)?
            .chars()
            .collect::<String>()
            .to_string();
        if name_string.contains("This license is copied below, and is also available with a FAQ") {
            /* This is the OFL text in a name table entry.
            It contains the term 'Reserved Font Name' in one of its clauses,
            so we will ignore this here. */
            continue;
        }

        let familyname = f
            .font()
            .localized_strings(NameId::FAMILY_NAME)
            .english_or_first()
            .ok_or(CheckError::Error("No name ID 1".to_string()))?
            .chars()
            .collect::<String>();
        #[allow(clippy::unwrap_used)]
        let re = Regex::new(r"with [Rr]eserved [Ff]ont [Nn]ame '(?<rfn>[^']*)'").unwrap();
        let matches = re.captures(&name_string);

        if matches.is_some() {
            #[allow(clippy::expect_used)]
            let rfn = &matches.expect("wont happen")["rfn"];
            if familyname.contains(rfn) {
                problems.push(Status::fail(
                    "rfn",
                    &format!(
                        "Name table entry contains \"Reserved Font Name\":\n\
                              \t\"{}\"\n\
                              \n\
                              This is bad except in a few specific rare cases.",
                        name_string
                    ),
                ));
            } else {
                problems.push(Status::warn(
                    "legacy-familyname",
                    &format!(
                        "Name table entry contains \"Reserved Font Name\" for a \
                              family name ('{}') that differs \
                              from the currently used family name ('{}'), \
                              which is fine.",
                        rfn, familyname
                    ),
                ));
            }
        }
    }
    return_result(problems)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {

    // --- CODETESTING ---
    use fontspector_checkapi::{CheckResult, Context, StatusCode};
    use serde_json::Map;
    use write_fonts::{tables::name::NameRecord, FontBuilder};

    macro_rules! TEST_FILE {
        ($fname:expr) => {
            Testable::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/resources/test/",
                $fname
            ))
            .unwrap()
        };
    }

    fn run_check(
        check: fontspector_checkapi::Check<'_>,
        font: Testable,
    ) -> std::option::Option<CheckResult> {
        let ctx: fontspector_checkapi::Context = Context {
            skip_network: false,
            network_timeout: Some(10),
            configuration: Map::new(),
            check_metadata: check.metadata(),
        };
        check.run(&TestableType::Single(&font), &ctx, None)
    }

    fn assert_pass(check_result: std::option::Option<CheckResult>) {
        let status = check_result.unwrap().worst_status();
        assert_eq!(status, StatusCode::Pass);
    }

    #[allow(dead_code)]
    fn assert_results_contain(
        check_result: std::option::Option<CheckResult>,
        severity: StatusCode,
        code: Option<String>,
    ) {
        let subresults = check_result.unwrap().subresults;
        assert!(subresults
            .iter()
            .any(|subresult| subresult.severity == severity && subresult.code == code));
    }

    fn set_name_entry(font: &mut Testable, platform: u16, encoding: u16, language: u16, nameid: NameId, new_string: String){
        use std::collections::BTreeSet;

        let f = TTF.from_testable(&font).unwrap();
        let name = f.font().name().unwrap();

        let new_record = NameRecord::new(
            platform,
            encoding,
            language,
            nameid,
            new_string.to_string().into(),
        );
        let mut new_records: BTreeSet<NameRecord> = name
            .name_record()
            .iter()
            .filter(|record| record.name_id() != nameid)
            .map(|r| {
                #[allow(clippy::unwrap_used)]
                NameRecord::new(
                    r.platform_id(),
                    r.encoding_id(),
                    r.language_id(),
                    r.name_id(),
                    r.string(name.string_data())
                        .unwrap()
                        .chars()
                        .collect::<String>()
                        .to_string()
                        .into(),
                )
            })
            .collect();
        new_records.insert(new_record);
        let new_nametable = Name::new(new_records);
        let new_bytes = FontBuilder::new()
            .add_table(&new_nametable)
            .unwrap()
            .copy_missing_tables(f.font())
            .build();

        font.contents = new_bytes;
    }

    // --- end of CODETESTING ---

    use crate::constants::OFL_BODY_TEXT;
    use write_fonts::tables::name::Name;
    use super::*;

    #[test]
    fn pass_with_good_font() {
        let font: Testable = TEST_FILE!("nunito/Nunito-Regular.ttf");
        assert_pass(run_check(super::name_rfn, font));
    }

    #[test]
    fn pass_with_good_font_containing_ofl_full_text() {
        /* The OFL text contains the term 'Reserved Font Name',
        which should not cause a FAIL: */
        let mut font: Testable = TEST_FILE!("nunito/Nunito-Regular.ttf");

        set_name_entry(&mut font,
                       3,  // PlatformID.WINDOWS
                       1,  // WindowsEncodingID.UNICODE_BMP
                       0x0409,  // WindowsLanguageID.ENGLISH_USA,
                       NameId::LICENSE_DESCRIPTION,
                       OFL_BODY_TEXT.to_string().into());
        assert_pass(run_check(super::name_rfn, font));
    }

/*
    #[test]
    fn fail_with_rfn_on_a_name_table_entry() {
        /* NOTE: This is not a real copyright statement.
           It is only meant to test the check. */
        let mut font: Testable = TEST_FILE!("nunito/Nunito-Regular.ttf");

        let with_nunito_rfn: String = "Copyright 2022 The Nunito Project Authors \
                                       (https://github.com/googlefonts/NunitoSans), \
                                       with Reserved Font Name Nunito.".to_string();
        set_name_entry(&mut font,
                       3,  // PlatformID.WINDOWS
                       1,  // WindowsEncodingID.UNICODE_BMP
                       0x0409,  // WindowsLanguageID.ENGLISH_USA,
                       NameId::VERSION_STRING,
                       with_nunito_rfn);
        assert_results_contain(run_check(super::name_rfn, font),
                               StatusCode::Fail, Some("rfn".to_string()));
    }

    #[test]
    fn warn_with_rfn_referencing_an_older_familyname_not_being_used_in_this_font_project(){
        /* NOTE: This is not a real copyright statement.
           It is only meant to test the check. */
        let mut font: Testable = TEST_FILE!("nunito/Nunito-Regular.ttf");
        let with_other_familyname_rfn: String = "Copyright 2022 The FooBar Project \
                                                 Authors (https://github.com/foo/bar), \
                                                 with Reserved Font Name FooBar.\
                                                 ".to_string();
        set_name_entry(&mut font,
                       3,  // PlatformID.WINDOWS
                       1,  // WindowsEncodingID.UNICODE_BMP
                       0x0409,  // WindowsLanguageID.ENGLISH_USA,
                       NameId::VERSION_STRING,
                       with_other_familyname_rfn);
        assert_results_contain(run_check(super::name_rfn, font),
                               StatusCode::Warn, Some("legacy-familyname".to_string()));
        // TODO: assert "(FooBar)" in msg
    }
*/
}
