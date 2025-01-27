use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use read_fonts::{types::NameId, TableProvider};

#[check(
    id = "opentype/name/postscript_vs_cff",
    rationale = "
        The PostScript name entries in the font's 'name' table should match
        the FontName string in the 'CFF ' table.

        The 'CFF ' table has a lot of information that is duplicated in other tables.
        This information should be consistent across tables, because there's
        no guarantee which table an app will get the data from.
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/2229",
    title = "CFF table FontName must match name table ID 6 (PostScript name)."
)]
fn postscript_vs_cff(t: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(t);
    skip!(
        !font.has_table(b"CFF "),
        "no-cff",
        "This check only applies to CFF fonts."
    );
    if font.font().cff()?.names().count() > 1 {
        return Ok(Status::just_one_fail(
            "cff-name-error",
            "Unexpected number of font names in CFF table.",
        ));
    }
    let cff_name = String::from_utf8_lossy(
        font.font()
            .cff()?
            .names()
            .get(0)
            .map_err(|e| CheckError::Error(format!("Error reading CFF table: {}", e)))?,
    );
    let name = font.get_name_entry_strings(NameId::POSTSCRIPT_NAME).next();
    if let Some(name) = name {
        if cff_name != name {
            return Ok(Status::just_one_fail(
                "ps-cff-name-mismatch",
                &format!(
                    "Name table PostScript name '{}' does not match CFF table FontName '{}'.",
                    name, cff_name,
                ),
            ));
        }
    }
    Ok(Status::just_one_pass())
}
