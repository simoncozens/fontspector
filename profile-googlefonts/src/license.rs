use std::sync::LazyLock;

use font_types::NameId;
use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;
use regex::Regex;
use skrifa::MetadataProvider;

#[allow(clippy::unwrap_used)]
static RFN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"with [Rr]eserved [Ff]ont [Nn]ame '(?<rfn>[^']*)'").unwrap());

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
        let matches = RFN_RE.captures(&name_string);

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
