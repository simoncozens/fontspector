#![allow(renamed_and_removed_lints, clippy::unwrap_used)]
include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));

use crate::checks::metadata::family_proto;
use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};


#[check(
    id = "googlefonts/metadata/can_render_samples",
    title = "Check samples can be rendered",
    rationale = "
        In order to prevent tofu from being seen on fonts.google.com, this check
        verifies that all samples specified by METADATA.pb can be properly
        rendered by the font.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3419",
    applies_to = "MDPB",
    implementation = "all"
)]
fn can_render_samples(c: &TestableCollection, context: &Context) -> CheckFnResult {
    let mdpb = c
        .get_file("METADATA.pb")
        .ok_or_else(|| CheckError::skip("no-mdpb", "No METADATA.pb file found"))?;
    let msg = family_proto(mdpb)?;
    let languages = msg.languages;
    if languages.is_empty() {
        skip!("no-languages", "No languages specified in METADATA.pb");
    }
    let fonts = msg
        .fonts
        .iter()
        .flat_map(|f| f.filename.as_ref())
        .flat_map(|f| c.get_file(f))
        .collect::<Vec<&Testable>>();
    if fonts.is_empty() {
        skip!("no-fonts", "No font files found in METADATA.pb");
    }
    let mut samples: Vec<(&str, String)> = vec![];
    for language in languages
        .iter()
        .flat_map(|l| google_fonts_languages::LANGUAGES.get(l.as_str()))
    {
        if let Some(st) = language.sample_text.as_ref() {
            if let Some(tester) = st.tester.as_ref() {
                let tester = tester
                    .to_string()
                    .replace("\n", " ")
                    .replace("\u{200b}", "");
                if let Some(name) = &language.name {
                    samples.push((name, tester));
                }
            }
        }
    }
    let mut problems = vec![];
    for font in fonts {
        // We could get all clever and use harfruzz here, but to honest,
        // the only way you get a .notdef is if you can't use cmap to map
        // the character to a glyph, so we'll just use that.
        let ttf = testfont!(font);
        let codepoints = ttf.codepoints(Some(context));
        for (langid, sample) in samples.iter() {
            if sample.chars().any(|c| !codepoints.contains(&(c as u32))) {
                problems.push(Status::fail(
                    "sample-text",
                    &format!(
                        "Font {} cannot render {} sample text: {}",
                        font.basename().unwrap_or_default(),
                        langid,
                        sample
                    ),
                ));
            }
        }
    }
    return_result(problems)
}
