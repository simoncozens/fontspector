use crate::checks::googlefonts::metadata::family_proto;
use fontspector_checkapi::prelude::*;

#[check(
    id = "googlefonts/metadata/reserved_font_name",
    rationale = "
        
        Unless an exception has been granted, we expect fonts on
        Google Fonts not to use the \"Reserved Font Name\" clause in their
        copyright information. This is because fonts with RFNs are difficult
        to modify in a libre ecosystem; anyone who forks the font (with a
        view to changing it) must first rename the font, which makes
        it difficult to pass changes back to upstream.

        There is also a potential licensing difficulty, in that Google Fonts
        web service subsets the font - a modification of the original - but
        then delivers the font with the same name, which could be seen as a
        violation of the reserved font name clause.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Copyright notice on METADATA.pb should not contain 'Reserved Font Name'.",
    implementation = "all"
)]
fn reserved_font_name(c: &TestableCollection, _context: &Context) -> CheckFnResult {
    let mdpb = c
        .get_file("METADATA.pb")
        .ok_or_else(|| CheckError::skip("no-mdpb", "No METADATA.pb file found"))?;

    let mut problems = vec![];
    let family_metadata = family_proto(mdpb)?;
    for font_metadata in family_metadata.fonts {
        let copyright = font_metadata.copyright();
        if copyright.contains("Reserved Font Name") {
            problems.push(Status::warn(
                "rfn",
                &format!("METADATA.pb: copyright field (\"{}\") contains \"Reserved Font Name\". This is an error except in a few specific rare cases.",
                copyright),
            ));
        }
    }
    return_result(problems)
}
