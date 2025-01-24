use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::types::NameId;

#[check(
    id = "name/char_restrictions",
    rationale = r#"
        The OpenType spec requires a subset of ASCII
        (any printable characters except "[]{}()<>/%") for
        POSTSCRIPT_NAME (nameID 6),
        POSTSCRIPT_CID_NAME (nameID 20), and
        an even smaller subset ("a-zA-Z0-9") for
        VARIATIONS_POSTSCRIPT_NAME_PREFIX (nameID 25).
    "#,
    proposal = "https://github.com/fonttools/fontbakery/issues/1718 and https://github.com/fonttools/fontbakery/issues/1663",
    title = "Are there disallowed characters in the NAME table?"
)]
fn char_restrictions(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems: Vec<Status> = vec![];
    let bad_char = |c: char| {
        !c.is_ascii()
            || c.is_ascii_control()
            || c == '['
            || c == ']'
            || c == '{'
            || c == '}'
            || c == '('
            || c == ')'
            || c == '<'
            || c == '>'
            || c == '/'
            || c == '%'
    };
    for record in f.get_name_entry_strings(NameId::POSTSCRIPT_NAME) {
        if record.contains(bad_char) {
            problems.push(Status::fail(
                "bad-string",
                &format!(
                    "Some namerecords with ID={} (NameID.POSTSCRIPT_NAME) '{}' contain disallowed characters.",
                    NameId::POSTSCRIPT_NAME.to_u16(),
                    record,
                ),
            ))
        }
    }
    for record in f.get_name_entry_strings(NameId::POSTSCRIPT_CID_NAME) {
        if record.contains(bad_char) {
            problems.push(Status::fail(
                "bad-string",
                &format!(
                    "Some namerecords with ID={} (NameID.POSTSCRIPT_CID_NAME) '{}' contain disallowed characters.",
                    NameId::POSTSCRIPT_CID_NAME.to_u16(),
                    record,
                ),
            ))
        }
    }
    for record in f.get_name_entry_strings(NameId::VARIATIONS_POSTSCRIPT_NAME_PREFIX) {
        if record.chars().any(|c| !c.is_ascii_alphanumeric()) {
            problems.push(Status::fail(
                "bad-string",
                &format!(
                    "Some namerecords with ID={} (NameID.VARIATIONS_POSTSCRIPT_NAME_PREFIX) '{}' contain disallowed characters.",
                    NameId::VARIATIONS_POSTSCRIPT_NAME_PREFIX.to_u16(),
                    record,
                ),
            ))
        }
    }
    if !problems.is_empty() {
        problems.push(Status::fail(
            "bad-strings",
            &format!("There are {} strings containing disallowed characters in the restricted name table entries", problems.len())
        ));
    }
    return_result(problems)
}
