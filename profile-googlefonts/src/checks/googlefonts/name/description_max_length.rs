use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use skrifa::raw::tables::name::NameId;

#[check(
    id = "googlefonts/name/description_max_length",
    rationale = "
        
        An old FontLab version had a bug which caused it to store copyright notices
        in nameID 10 entries.

        In order to detect those and distinguish them from actual legitimate usage of
        this name table entry, we expect that such strings do not exceed a reasonable
        length of 200 chars.

        Longer strings are likely instances of the FontLab bug.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Description strings in the name table must not exceed 200 characters."
)]
fn description_max_length(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    if f.get_name_entry_strings(NameId::DESCRIPTION)
        .any(|s| s.len() > 200)
    {
        return Ok(Status::just_one_warn(
            "too-long",
            "A few name table entries with ID=10 (NameID.DESCRIPTION) are longer than 200 characters. Please check whether those entries are copyright notices mistakenly stored in the description string entries by a bug in an old FontLab version. If that's the case, then such copyright notices must be removed from these entries.",
        ));
    }
    Ok(Status::just_one_pass())
}
