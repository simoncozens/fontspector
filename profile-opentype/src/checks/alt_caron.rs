use std::collections::HashMap;

use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::tables::glyf::Glyph;
use skrifa::MetadataProvider;

const CARON_CODEPOINTS: [u32; 4] = [
    0x013D, // LATIN CAPITAL LETTER L WITH CARON
    0x010F, // LATIN SMALL LETTER D WITH CARON
    0x013E, // LATIN SMALL LETTER L WITH CARON
    0x0165, // LATIN SMALL LETTER T WITH CARON
];

const WRONG_CARON_MARKS: [u32; 2] = [
    0x02C7, // CARON
    0x030C, // COMBINING CARON
];

const BAD_CARON_MARKS: [u32; 4] = [
    0x002C, // COMMA
    0x2019, // RIGHT SINGLE QUOTATION MARK
    0x201A, // SINGLE LOW-9 QUOTATION MARK
    0x0027, // APOSTROPHE
];

fn mangle_name(glyph: &str) -> String {
    glyph
        .replace(".case", "")
        .replace(".uc", "")
        .replace(".sc", "")
}

#[check(
    id = "alt_caron",
    title = "Check accent of Lcaron, dcaron, lcaron, tcaron",
    rationale = "
        Lcaron, dcaron, lcaron, tcaron should NOT be composed with quoteright
        or quotesingle or comma or caron(comb). It should be composed with a
        distinctive glyph which doesn't look like an apostrophe.

        Source:
        https://ilovetypography.com/2009/01/24/on-diacritics/
        http://diacritics.typo.cz/index.php?id=5
        https://www.typotheque.com/articles/lcaron
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3308"
)]
fn alt_caron(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    let glyphname_to_codepoint: HashMap<String, u32> = f
        .codepoints()
        .iter()
        .copied()
        .map(|codepoint| (f.glyph_name_for_unicode_synthesise(codepoint), codepoint))
        .collect();
    let charmap = f.font().charmap();
    for caron in CARON_CODEPOINTS {
        if let Some(gid) = charmap.map(caron) {
            if let Ok(Some(glyph)) = f.get_glyf_glyph(gid) {
                match glyph {
                    Glyph::Simple(_) => {
                        let name = f.glyph_name_for_id_synthesise(gid);
                        problems.push(Status::warn("decomposed-outline",&format!("{} is decomposed and therefore could not be checked. Please check manually.", name)));
                    }
                    Glyph::Composite(composite) => {
                        if composite.components().count() == 1 {
                            problems.push(Status::warn("single-compoents", &format!("{} is composed of a single component and therefore could not be checked. Please check manually.", f.glyph_name_for_id_synthesise(gid))));
                        } else {
                            for component in composite.components() {
                                let comp_name =
                                    mangle_name(&f.glyph_name_for_id_synthesise(component.glyph));
                                if let Some(codepoint) = glyphname_to_codepoint.get(&comp_name) {
                                    if BAD_CARON_MARKS.contains(codepoint) {
                                        problems.push(Status::warn(
                                            "bad-mark",
                                            &format!(
                                                "{} uses component: {}",
                                                f.glyph_name_for_id_synthesise(gid),
                                                comp_name
                                            ),
                                        ));
                                    } else if WRONG_CARON_MARKS.contains(codepoint) {
                                        problems.push(Status::fail(
                                            "wrong-mark",
                                            &format!(
                                                "{} uses component: {}",
                                                f.glyph_name_for_id_synthesise(gid),
                                                comp_name
                                            ),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    return_result(problems)
}
