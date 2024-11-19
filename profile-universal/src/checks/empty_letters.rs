use std::collections::HashSet;

use fontspector_checkapi::{
    constants::{ALL_HANGUL_SYLLABLES_CODEPOINTS, MODERN_HANGUL_SYLLABLES_CODEPOINTS},
    pens::AnythingPen,
    prelude::*,
    testfont, FileTypeConvert, TestFont, DEFAULT_LOCATION,
};
use skrifa::{GlyphId, MetadataProvider};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

const INVISIBLE_LETTERS: [u32; 4] = [0x115F, 0x1160, 0x3164, 0xFFA0];

fn is_letter(codepoint: u32) -> bool {
    char::from_u32(codepoint)
        .map(|c| {
            matches!(
                c.general_category(),
                GeneralCategory::LowercaseLetter
                    | GeneralCategory::ModifierLetter
                    | GeneralCategory::OtherLetter
                    | GeneralCategory::TitlecaseLetter
                    | GeneralCategory::UppercaseLetter
            )
        })
        .unwrap_or(false)
}
#[check(
    id = "empty_letters",
    rationale = "
        Font language, script, and character set tagging approaches typically have an
        underlying assumption that letters (i.e. characters with Unicode general
        category 'Ll', 'Lm', 'Lo', 'Lt', or 'Lu', which includes CJK ideographs and
        Hangul syllables) with entries in the 'cmap' table have glyphs with ink (with
        a few exceptions, notably the four Hangul \"filler\" characters: U+115F, U+1160,
        U+3164, U+FFA0).

        This check is intended to identify fonts in which such letters have been mapped
        to empty glyphs (typically done as a form of subsetting). Letters with empty
        glyphs should have their entries removed from the 'cmap' table, even if the
        empty glyphs are left in place (e.g. for CID consistency).

        The check will yield only a WARN if the blank glyph maps to a character in the
        range of Korean hangul syllable code-points, which are known to be used by font
        designers as a workaround to undesired behavior from InDesign's Korean IME
        (Input Method Editor).

        More details available at https://github.com/fonttools/fontbakery/issues/2894
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/2460",
    title = "Letters in font have glyphs that are not empty?"
)]
fn empty_letters(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let blank_ok_set: HashSet<u32> = ALL_HANGUL_SYLLABLES_CODEPOINTS
        .collect::<HashSet<u32>>()
        .difference(
            &MODERN_HANGUL_SYLLABLES_CODEPOINTS
                .into_iter()
                .collect::<HashSet<u32>>(),
        )
        .copied()
        .collect();
    let mut num_blank_hangul = 0;
    let mut problems = vec![];
    for (codepoint, gid) in f.font().charmap().mappings() {
        if blank_ok_set.contains(&codepoint) && is_blank_glyph(&f, gid)? {
            num_blank_hangul += 1;
            continue;
        }
        if !INVISIBLE_LETTERS.contains(&codepoint)
            && is_letter(codepoint)
            && is_blank_glyph(&f, gid)?
        {
            problems.push(Status::fail(
                "empty-letter",
                &format!(
                    "U+{:04X} should be visible, but its glyph ('{}') is empty.",
                    codepoint,
                    f.glyph_name_for_unicode_synthesise(codepoint)
                ),
            ));
        }
    }
    if num_blank_hangul > 0 {
        problems.push(Status::warn(
            "empty-hangul-letter",
            &format!("Found {} empty hangul glyph(s).", num_blank_hangul),
        ));
    }
    return_result(problems)
}

fn is_blank_glyph(f: &TestFont, gid: GlyphId) -> Result<bool, CheckError> {
    let mut pen = AnythingPen::default();
    f.draw_glyph(gid, &mut pen, DEFAULT_LOCATION)?;
    Ok(!pen.anything())
}
