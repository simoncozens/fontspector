use fontspector_checkapi::{constants::OutlineType, prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;
use skrifa::FontRef;

use crate::utils::build_expected_font;

#[check(
    id = "googlefonts/weightclass",
    rationale = "
        
        Google Fonts expects variable fonts, static ttfs and static otfs to have
        differing OS/2 usWeightClass values.

        - For Variable Fonts, Thin-Black must be 100-900

        - For static ttfs, Thin-Black can be 100-900 or 250-900

        - For static otfs, Thin-Black must be 250-900

        If static otfs are set lower than 250, text may appear blurry in
        legacy Windows applications.

        Glyphsapp users can change the usWeightClass value of an instance by adding
        a 'weightClass' customParameter.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Check the OS/2 usWeightClass is appropriate for the font's best SubFamily name."
)]
fn googlefonts_weightclass(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    let value = f.font().os2()?.us_weight_class();
    let expected_names = build_expected_font(&f, &[])?;
    let expected_value = FontRef::new(&expected_names)?.os2()?.us_weight_class();
    let style_name = f.best_subfamilyname().unwrap_or("Regular".to_string());
    if f.is_variable_font() {
        if value != expected_value {
            problems.push(Status::fail(
                "bad-value",
                &format!(
                    "Best SubFamily name is '{}'. Expected OS/2 usWeightClass is {}, got {}.",
                    style_name, expected_value, value
                ),
            ))
        }
    } else if style_name.contains("Thin") {
        if f.outline_type() == OutlineType::TrueType && ![100, 250].contains(&value) {
            problems.push(Status::fail(
                "bad-value",
                &format!(
                    "Best SubFamily name is '{}'. Expected OS/2 usWeightClass is {}, got {}.",
                    style_name, expected_value, value
                ),
            ))
        }
        if f.outline_type() == OutlineType::CFF && value != 250 {
            problems.push(Status::fail(
                "bad-value",
                &format!(
                    "Best SubFamily name is '{}'. Expected OS/2 usWeightClass is {}, got {}.",
                    style_name, 250, value
                ),
            ))
        }
    } else if style_name.contains("ExtraLight") {
        if f.outline_type() == OutlineType::TrueType && ![200, 275].contains(&value) {
            problems.push(Status::fail(
                "bad-value",
                &format!(
                    "Best SubFamily name is '{}'. Expected OS/2 usWeightClass is {}, got {}.",
                    style_name, expected_value, value
                ),
            ))
        }
        if f.outline_type() == OutlineType::CFF && value != 275 {
            problems.push(Status::fail(
                "bad-value",
                &format!(
                    "Best SubFamily name is '{}'. Expected OS/2 usWeightClass is {}, got {}.",
                    style_name, 275, value
                ),
            ))
        }
    } else if value != expected_value {
        problems.push(Status::fail(
            "bad-value",
            &format!(
                "Best SubFamily name is '{}'. Expected OS/2 usWeightClass is {}, got {}.",
                style_name, expected_value, value
            ),
        ))
    }
    return_result(problems)
}
