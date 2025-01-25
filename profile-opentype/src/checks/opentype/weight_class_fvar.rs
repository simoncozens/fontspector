use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use read_fonts::TableProvider;


#[check(
    id = "opentype/weight_class_fvar",
    rationale = "According to Microsoft's OT Spec the OS/2 usWeightClass should match the fvar default value.",
    proposal = "https://github.com/googlefonts/gftools/issues/477",
    title = "Checking if OS/2 usWeightClass matches fvar."
)]
fn weight_class_fvar(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.is_variable_font(), "not-variable", "Not a variable font");
    let fvar_value = f
        .axis_ranges()
        .find(|(tag, _, _, _)| tag == "wght")
        .map(|(_, _, default, _)| default)
        .ok_or(CheckError::skip("no-wght", "No 'wght' axis"))?;
    let os2_value = f
        .font()
        .os2()
        .map_err(|_| CheckError::skip("no-os2", "No OS/2 table"))?
        .us_weight_class();
    if os2_value != fvar_value as u16 {
        return Ok(Status::just_one_fail(
            "bad-weight-class",
            &format!(
                "OS/2 usWeightClass is {}, but fvar default is {}",
                os2_value, fvar_value
            ),
        ));
    }

    Ok(Status::just_one_pass())
}
