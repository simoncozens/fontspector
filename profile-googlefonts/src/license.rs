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
    proposal="https://github.com/fonttools/fontbakery/issues/1380",
)]
fn name_rfn(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];

    let records = f.font().name()?.name_record();
    let str_data = f.font().name()?.string_data();

    for name in records {
        let name_string = name.string(str_data)?
                              .chars()
                              .collect::<String>()
                              .to_string();
        if name_string.contains(
            "This license is copied below, and is also available with a FAQ") {
            /* This is the OFL text in a name table entry.
               It contains the term 'Reserved Font Name' in one of its clauses,
               so we will ignore this here. */
            continue;
        }

        let familyname = f.font()
                          .localized_strings(NameId::FAMILY_NAME)
                          .english_or_first()
                          .ok_or(CheckError::Error("No name ID 1".to_string()))?
                          .chars()
                          .collect::<String>();
        let re = Regex::new(r"with [Rr]eserved [Ff]ont [Nn]ame '(?<rfn>[^']*)'").unwrap();
        let matches = re.captures(&name_string);

        if matches.is_some() {
            let rfn = &matches.expect("wont happen")["rfn"];
            if familyname.contains(rfn) {
                problems.push(Status::fail("rfn",
                    &format!("Name table entry contains \"Reserved Font Name\":\n\
                              \t\"{}\"\n\
                              \n\
                              This is bad except in a few specific rare cases.",
                              name_string)));
            }
            else {
                problems.push(Status::warn("legacy-familyname",
                    &format!("Name table entry contains \"Reserved Font Name\" for a \
                              family name ('{}') that differs \
                              from the currently used family name ('{}'), \
                              which is fine.",
                              rfn, familyname)));
            }
        }
    }
    return_result(problems)
}

#[cfg(test)]
mod tests {

// --- CODETESTING ---
    use fontspector_checkapi::{Context, StatusCode, CheckResult};
    use serde_json::Map;

    macro_rules! TEST_FILE {($fname:expr) => (
        Testable::new(
            concat!(env!("CARGO_MANIFEST_DIR"), "/resources/test/", $fname)
        ).unwrap()
    )}

    fn run_check(check: fontspector_checkapi::Check<'_>, font: Testable) -> std::option::Option<CheckResult> {
        let ctx: fontspector_checkapi::Context = Context {
            skip_network: false,
            network_timeout: Some(10),
            configuration: Map::new(),
        };
        let section: &str = "Licensing Checks";
        check.run(&TestableType::Single(&font), &ctx, section)
    }

    fn assert_pass(check_result: std::option::Option<CheckResult>) {
        let status = check_result.unwrap().worst_status();
        assert_eq!(status, StatusCode::Pass);
    }


    fn assert_results_contain(check_result: std::option::Option<CheckResult>, severity: StatusCode, code: Option<String>) {
        let subresults = check_result.unwrap().subresults;
        assert!(subresults.iter().any(|subresult| subresult.severity == severity && subresult.code == code));
    }

// --- end of CODETESTING ---

    use super::*;
    //use crate::constants::OFL_BODY_TEXT;

    #[test]
    fn pass_with_good_font(){
        let font: Testable = TEST_FILE!("nunito/Nunito-Regular.ttf");
        assert_pass(run_check(super::name_rfn, font));
    }

    #[test]
    fn pass_with_good_font_containing_ofl_full_text(){
        /* The OFL text contains the term 'Reserved Font Name',
           which should not cause a FAIL: */
        let font: Testable = TEST_FILE!("nunito/Nunito-Regular.ttf");

/* FIXME:
        let f = testfont!(font);
        f.font().name().setName(
            OFL_BODY_TEXT,
            NameId::LICENSE_DESCRIPTION,
            3,      // PlatformID.WINDOWS
            1,      // WindowsEncodingID.UNICODE_BMP
            0x0409, // WindowsLanguageID.ENGLISH_USA
        );
*/
        assert_pass(run_check(super::name_rfn, font));
    }
}

/* TODO:

    # NOTE: This is not a real copyright statement. It is only meant to test the check.
    with_nunito_rfn = (
        "Copyright 2022 The Nunito Project Authors"
        " (https://github.com/googlefonts/NunitoSans),"
        " with Reserved Font Name Nunito."
    )
    ttFont["name"].setName(
        with_nunito_rfn,
        NameID.VERSION_STRING,
        PlatformID.WINDOWS,
        WindowsEncodingID.UNICODE_BMP,
        WindowsLanguageID.ENGLISH_USA,
    )
    assert_results_contain(
        check(ttFont),
        FAIL,
        "rfn",
        'with "Reserved Font Name Nunito" on a name table entry...',
    )

    # NOTE: This is not a real copyright statement. It is only meant to test the check.
    with_other_familyname_rfn = (
        "Copyright 2022 The FooBar Project Authors"
        " (https://github.com/foo/bar),"
        " with Reserved Font Name FooBar."
    )
    ttFont["name"].setName(
        with_other_familyname_rfn,
        NameID.VERSION_STRING,
        PlatformID.WINDOWS,
        WindowsEncodingID.UNICODE_BMP,
        WindowsLanguageID.ENGLISH_USA,
    )
    msg = assert_results_contain(
        check(ttFont),
        WARN,
        "legacy-familyname",
        'with "Reserved Font Name" that references an older'
        " familyname not being used in this font project...",
    )
    assert "(FooBar)" in msg

    } */
