use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::{tables::gdef::GlyphClassDef, TableProvider};
use skrifa::MetadataProvider;

const LEGACY_ACCENTS: [u32; 13] = [
    0x00A8, // DIAERESIS
    0x02D9, // DOT ABOVE
    0x0060, // GRAVE ACCENT
    0x00B4, // ACUTE ACCENT
    0x02DD, // DOUBLE ACUTE ACCENT
    0x02C6, // MODIFIER LETTER CIRCUMFLEX ACCENT
    0x02C7, // CARON
    0x02D8, // BREVE
    0x02DA, // RING ABOVE
    0x02DC, // SMALL TILDE
    0x00AF, // MACRON
    0x00B8, // CEDILLA
    0x02DB, // OGONEK
];

#[check(
    id = "legacy_accents",
    rationale = "
        Legacy accents should not have anchors and should have positive width.
        They are often used independently of a letter, either as a placeholder
        for an expected combined mark+letter combination in MacOS, or separately.
        For instance, U+00B4 (ACUTE ACCENT) is often mistakenly used as an apostrophe,
        U+0060 (GRAVE ACCENT) is used in Markdown to notify code blocks,
        and ^ is used as an exponential operator in maths.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4310",
    title = "Check that legacy accents aren't used in composite glyphs."
)]
fn legacy_accents(f: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    let mut problems = vec![];
    let hmtx = font.font().hmtx()?;

    let charmap = font.font().charmap();
    for gid in LEGACY_ACCENTS.iter().flat_map(|c| charmap.map(*c)) {
        if hmtx.advance(gid).unwrap_or(0) == 0 {
            problems.push(Status::fail(
                "legacy-accents-width",
                &format!(
                    "Width of legacy accent \"{}\" is zero; should be positive",
                    font.glyph_name_for_id_synthesise(gid)
                ),
            ));
        }
        if font.gdef_class(gid) == GlyphClassDef::Mark {
            problems.push(Status::fail(
                "legacy-accents-gdef",
                &format!(
                    "Legacy accent \"{}\" is defined in GDEF as a mark (class 3).",
                    font.glyph_name_for_id_synthesise(gid)
                ),
            ));
        }
    }
    return_result(problems)
}
