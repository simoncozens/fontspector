use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};

#[check(
    id = "smallcaps_before_ligatures",
    rationale = "
        OpenType small caps should be defined before ligature lookups to ensure
        proper functionality.

        Rainer Erich Scheichelbauer (a.k.a. MekkaBlue) pointed out in a tweet
        (https://twitter.com/mekkablue/status/1297486769668132865) that the ordering
        of small caps and ligature lookups can lead to bad results such as the example
        he provided of the word \"WAFFLES\" in small caps, but with an unfortunate
        lowercase ffl ligature substitution.
	
        This check attempts to detect this kind of mistake.",
    proposal = "https://github.com/fonttools/fontbakery/issues/3020",
    title = "Ensure 'smcp' (small caps) lookups are defined before ligature lookups in the 'GSUB' table."
)]
fn smallcaps_before_ligatures(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    // Skip if no smcp
    let smcp_lookups = f
        .feature_records(true)
        .filter(|(r, _l)| r.feature_tag() == "smcp")
        .flat_map(|(_r, l)| l)
        .flat_map(|l| l.lookup_list_indices())
        .collect::<Vec<_>>();
    let liga_lookups = f
        .feature_records(true)
        .filter(|(r, _l)| r.feature_tag() == "liga")
        .flat_map(|(_r, l)| l)
        .flat_map(|l| l.lookup_list_indices())
        .collect::<Vec<_>>();
    skip!(smcp_lookups.is_empty(), "no-smcp", "No smcp feature");
    skip!(liga_lookups.is_empty(), "no-liga", "No liga feature");
    #[allow(clippy::unwrap_used)] // We know that the vecs are not empty
    let first_smcp_lookup = smcp_lookups.iter().min().unwrap();
    #[allow(clippy::unwrap_used)] // We know that the vecs are not empty
    let first_liga_lookup = liga_lookups.iter().min().unwrap();
    if first_smcp_lookup < first_liga_lookup {
        return Ok(Status::just_one_pass());
    }
    return Ok(Status::just_one_fail(
        "feature-ordering",
        "'smcp' lookups are not defined before 'liga' lookups.",
    ));
}
