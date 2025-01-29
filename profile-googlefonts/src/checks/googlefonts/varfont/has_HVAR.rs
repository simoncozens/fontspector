use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};

#[check(
    id = "googlefonts/varfont/has_HVAR",
    rationale = "
        
        Not having a HVAR table can lead to costly text-layout operations on some
        platforms, which we want to avoid.

        So, all variable fonts on the Google Fonts collection should have an HVAR
        with valid values.

        More info on the HVAR table can be found at:
        https://docs.microsoft.com/en-us/typography/opentype/spec/otvaroverview#variation-data-tables-and-miscellaneous-requirements
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/2119",
    title = "Check that variable fonts have an HVAR table."
)]
fn has_HVAR(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(
        !f.is_variable_font(),
        "variable-font",
        "Font is not a variable font."
    );
    Ok(if f.has_table(b"HVAR") {
        Status::just_one_pass()
    } else {
        Status::just_one_fail("lacks-HVAR",
            "All variable fonts on the Google Fonts collection must have a properly set HVAR table in order to avoid costly text-layout operations on certain platforms."
        )
    })
}
