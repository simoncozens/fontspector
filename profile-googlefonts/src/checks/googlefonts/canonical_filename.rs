use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use google_fonts_axisregistry::build_filename;

#[check(
    id = "googlefonts/canonical_filename",
    rationale = "
        
        A font's filename must be composed as \"<familyname>-<stylename>.ttf\":

        - Nunito-Regular.ttf

        - Oswald-BoldItalic.ttf


        Variable fonts must list the axis tags in alphabetical order in
        square brackets and separated by commas:

        - Roboto[wdth,wght].ttf

        - Familyname-Italic[wght].ttf
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Checking file is named canonically."
)]
fn canonical_filename(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let current_filename = t.basename().unwrap_or_default();
    let expected_filename = build_filename(f.font(), &t.extension().unwrap_or_default());
    Ok(if current_filename != expected_filename {
        Status::just_one_fail(
            "bad-filename",
            &format!(
                "Expected \"{}\". Got \"{}\".",
                expected_filename, current_filename
            ),
        )
    } else {
        Status::just_one_pass()
    })
}
