use fontspector_checkapi::{CheckError, TestFont};
use google_fonts_axisregistry::{build_fvar_instances, build_name_table, build_stat};
use skrifa::FontRef;

pub(crate) fn build_expected_font<'a>(
    font: &'a TestFont,
    siblings: &[&'a TestFont],
) -> Result<Vec<u8>, CheckError> {
    let mut new_binary = build_name_table(font.font(), None, None, &[], None)
        .map_err(|e| CheckError::Error(e.to_string()))?;
    let mut newfont = FontRef::new(&new_binary)?;
    if font.is_variable_font() {
        new_binary =
            build_fvar_instances(newfont, None).map_err(|e| CheckError::Error(e.to_string()))?;
        // And again...
        newfont = FontRef::new(&new_binary)?;
        let siblings: Vec<FontRef<'_>> = siblings.iter().map(|x| x.font()).collect();
        new_binary =
            build_stat(newfont, &siblings).map_err(|e| CheckError::Error(e.to_string()))?;
    }
    Ok(new_binary)
}
