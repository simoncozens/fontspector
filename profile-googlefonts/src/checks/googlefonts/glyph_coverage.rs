use crate::checks::googlefonts::metadata::family_proto;
use fontspector_checkapi::{prelude::*, skip, FileTypeConvert};
use google_fonts_glyphsets::get_coverage;

#[check(
    id = "googlefonts/glyph_coverage",
    rationale = "
        
        Google Fonts expects that fonts in its collection support at least the minimal
        set of characters defined in the `GF-latin-core` glyph-set.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/2488",
    title = "Check Google Fonts glyph coverage.",
    implementation = "all"
)]
fn glyph_coverage(c: &TestableCollection, context: &Context) -> CheckFnResult {
    let required_glyphset = if c
        .get_file("METADATA.pb")
        .and_then(|mdpb| family_proto(mdpb).ok())
        .map(|msg| msg.primary_script().to_string())
        .is_some()
    {
        "GF_Latin_Kernel"
    } else {
        "GF_Latin_Core"
    };
    let mut problems = vec![];

    skip!(
        context
            .configuration
            .get("icon_font")
            .and_then(|x| x.as_bool())
            .unwrap_or_default(),
        "icon-font",
        "This is an icon font or a symbol font."
    );

    for f in c.iter().flat_map(|t| TTF.from_testable(t)) {
        let codepoints = f.codepoints(Some(context));
        #[allow(clippy::unwrap_used)]
        // A static key lookup of one or another key we know to be in there
        let coverage = get_coverage(&codepoints, required_glyphset).unwrap();
        if !coverage.missing.is_empty() {
            let missing = coverage
                .missing
                .iter()
                .map(|c| format!("0x{:04X}", c,))
                .collect::<Vec<String>>();
            problems.push(Status::fail(
                "missing-codepoints",
                &format!(
                    "{} missing required codepoints:\n\n{}",
                    f.filename.as_os_str().to_string_lossy(),
                    bullet_list(context, missing)
                ),
            ));
        }
    }
    return_result(problems)
}
