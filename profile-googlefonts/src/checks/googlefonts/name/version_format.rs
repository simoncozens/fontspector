use std::sync::LazyLock;

use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;
use regex::Regex;
use skrifa::string::StringId;

#[allow(clippy::unwrap_used)]
static VALID_VERSION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Version\s0*[1-9][0-9]*\.\d+").unwrap());

#[check(
    id = "googlefonts/name/version_format",
    rationale = "
        
        For Google Fonts, the version string must be in the format \"Version X.Y\".
        The version number must be greater than or equal to 1.000. (Additional
        information following the numeric version number is acceptable.)
        The \"Version \" prefix is a recommendation given by the OpenType spec.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Version format is correct in 'name' table?"
)]
fn version_format(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    let name = f.font().name()?;
    let mut num_entries: u8 = 0;
    for record in name.name_record().iter() {
        let string = record.string(name.string_data())?;
        if record.name_id() == StringId::VERSION_STRING {
            num_entries += 1;

            let entry = string.to_string();
            let matches = VALID_VERSION_RE.captures(&entry);
            if matches.is_none() {
                problems.push(Status::fail(
                    "bad-version-strings",
                    &format!(
                        "The NameID.VERSION_STRING (nameID=5) value must follow\
                          the pattern \"Version X.Y\" with X.Y greater than or\
                          equal to 1.000.\
                         The \"Version\" prefix is a recommendation given by the\
                          OpenType spec. Current version string is: \"{}\"",
                        string,
                    ),
                ));
            }
        }
    }

    if num_entries == 0 {
        return Ok(Status::just_one_fail(
            "no-version-string",
            "Font lacks a VERSION_STRING (nameID=5) entry",
        ));
    }

    return_result(problems)
}
