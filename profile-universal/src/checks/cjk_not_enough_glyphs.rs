use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert, TestFont};
use read_fonts::TableProvider;

const CJK_CODEPAGE_BITS: [u8; 5] = [17, 18, 19, 20, 21];

fn is_claiming_to_be_cjk_font(f: &TestFont) -> bool {
    if let Ok(os2) = f.font().os2() {
        if let Some(codepages) = os2.ul_code_page_range_1() {
            for bit in CJK_CODEPAGE_BITS.iter() {
                if codepages & (1 << bit) != 0 {
                    return true;
                }
            }
        }
        // Urgh this is messy
        if (os2.ul_unicode_range_1() & (1 << 28)) != 0 || // Jamo
           (os2.ul_unicode_range_2() & (1 << (49-32))) != 0 || // Katakana
           (os2.ul_unicode_range_2() & (1 << (50-32))) != 0 || // Hiragana
            (os2.ul_unicode_range_2() & (1 << (51-32))) != 0 || // Bopomofo
            (os2.ul_unicode_range_2() & (1 << (52-32))) != 0 || // Hangul Compatibility Jamo
            (os2.ul_unicode_range_2() & (1 << (54-32))) != 0 || // Enclosed CJK Letters And Months
            (os2.ul_unicode_range_2() & (1 << (55-32))) != 0 || // CJK Compatibility
            (os2.ul_unicode_range_2() & (1 << (56-32))) != 0 || // Hangul Syllables
            (os2.ul_unicode_range_2() & (1 << (59-32))) != 0 || // CJK Unified Ideographs
            (os2.ul_unicode_range_2() & (1 << (61-32))) != 0
        // CJK Strokes
        {
            return true;
        }
        false
    } else {
        false
    }
}

#[check(
    id = "cjk_not_enough_glyphs",
    rationale = "
        Kana has 150 characters and it's the smallest CJK writing system.

        If a font contains less CJK glyphs than this writing system, we inform the
        user that some glyphs may be encoded incorrectly.
    ",
    title = "Any CJK font should contain at least a minimal set of 150 CJK characters.",
    proposal = "https://github.com/fonttools/fontbakery/pull/3214"
)]
fn cjk_not_enough_glyphs(f: &Testable, context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    skip!(
        !is_claiming_to_be_cjk_font(&font),
        "not-cjk",
        "Not a CJK font."
    );
    let cjk_glyphs: Vec<_> = font.cjk_codepoints(Some(context)).collect();
    let cjk_glyph_count = cjk_glyphs.len();
    Ok(if cjk_glyph_count > 0 && cjk_glyph_count < 150 {
        let num_cjk_glyphs = if cjk_glyph_count == 1 {
            "There is only one CJK glyph"
        } else {
            &format!("There are only {} CJK glyphs", cjk_glyph_count)
        };
        Status::just_one_warn(
            "cjk-not-enough-glyphs",
            &format!(
                "{} when there needs to be at least 150 in order to support the smallest CJK writing system, Kana.\nThe following CJK glyphs were found:\n{}\nPlease check that these glyphs have the correct unicodes.",
                num_cjk_glyphs,
                bullet_list(context, cjk_glyphs)
            ),
        )
    } else {
        Status::just_one_pass()
    })
}
