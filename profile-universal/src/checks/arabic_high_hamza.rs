use fontspector_checkapi::{
    pens::AreaPen, prelude::*, skip, testfont, FileTypeConvert, DEFAULT_LOCATION,
};
use read_fonts::tables::gdef::GlyphClassDef;
use skrifa::MetadataProvider;

const ARABIC_LETTER_HAMZA: u32 = 0x0621;
const ARABIC_LETTER_HIGH_HAMZA: u32 = 0x0675;

#[check(
    id = "arabic_high_hamza",
    title = "Check that glyph for U+0675 ARABIC LETTER HIGH HAMZA is not a mark.",
    rationale = "
        Many fonts incorrectly treat ARABIC LETTER HIGH HAMZA (U+0675) as a variant of
        ARABIC HAMZA ABOVE (U+0654) and make it a combining mark of the same size.

        But U+0675 is a base letter and should be a variant of ARABIC LETTER HAMZA
        (U+0621) but raised slightly above baseline.

        Not doing so effectively makes the font useless for Jawi and
        possibly Kazakh as well.
    ",
    proposal = "https://github.com/googlefonts/fontbakery/issues/4290"
)]
fn arabic_high_hamza(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let codepoints = f.codepoints(Some(context));
    let mut problems = vec![];
    skip!(
        !codepoints.contains(&ARABIC_LETTER_HIGH_HAMZA)
            || !codepoints.contains(&ARABIC_LETTER_HAMZA),
        "glyphs-missing",
        "This check will only run on fonts that have both glyphs U+0621 and U+0675"
    );

    #[allow(clippy::unwrap_used)] // We just tested for it
    let high_hamza = f.font().charmap().map(ARABIC_LETTER_HIGH_HAMZA).unwrap();
    if f.gdef_class(high_hamza) == GlyphClassDef::Mark {
        problems.push(Status::fail(
            "mark-in-gdef",
            &format!(
                "{} is defined in GDEF as a mark (class 3).",
                f.glyph_name_for_id_synthesise(high_hamza)
            ),
        ))
    }
    let mut pen = AreaPen::new();
    f.draw_glyph(high_hamza, &mut pen, DEFAULT_LOCATION)?;
    let high_hamza_area = pen.area();

    #[allow(clippy::unwrap_used)] // We just tested for it
    let hamza = f.font().charmap().map(ARABIC_LETTER_HAMZA).unwrap();
    let mut pen = AreaPen::new();
    f.draw_glyph(hamza, &mut pen, DEFAULT_LOCATION)?;
    let hamza_area = pen.area();

    if ((high_hamza_area - hamza_area) / hamza_area).abs() > 0.1 {
        problems.push(Status::fail(
            "glyph-area",
            "The arabic letter high hamza (U+0675) should have roughly the same size the arabic letter hamza (U+0621), but a different glyph outline area was detected.",
        ))
    }

    return_result(problems)
}
