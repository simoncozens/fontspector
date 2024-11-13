use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;

const NANOEMOJI_ADVICE : &str = "You can do it by using the maximum_color tool provided by the nanoemoji project:\nhttps://github.com/googlefonts/nanoemoji";

#[check(
    id = "colorfont_tables",
    rationale = "
        COLR v0 fonts are widely supported in most browsers so they do not require
        an SVG color table. However, some environments (e.g. Safari, Adobe apps)
        do not currently support COLR v1 so we need to add an SVG table to these fonts,
        except in the case of variable fonts, since SVG does not support
        OpenType Variations.

        To automatically generate compatible SVG/COLR tables,
        run the maximum_color tool in nanoemoji:
        https://github.com/googlefonts/nanoemoji
    ",
    proposal = "https://googlefonts.github.io/gf-guide/color.html and https://github.com/fonttools/fontbakery/issues/3886 and https://github.com/fonttools/fontbakery/issues/3888 and https://github.com/fonttools/fontbakery/pull/3889 and https://github.com/fonttools/fontbakery/issues/4131",
    title = "Ensure font has the expected color font tables."
)]
fn colorfont_tables(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    if f.has_table(b"COLR") {
        if f.font().colr()?.version() == 0 && f.has_table(b"SVG ") {
            problems.push(Status::fail("drop-svg", "Font has a COLR v0 table, which is already widely supported, so the SVG table isn't needed."));
        } else if f.font().colr()?.version() == 1 && !f.has_table(b"SVG ") && !f.is_variable_font()
        {
            problems.push(Status::fail("add-svg", 
                &format!(
            "Font has COLRv1 but no SVG table; for CORLv1, we require that an SVG table is present to support environments where the former is not supported yet.\n{}", NANOEMOJI_ADVICE)));
        }
    }

    if f.has_table(b"SVG ") {
        if f.is_variable_font() {
            problems.push(Status::fail("variable-svg",
                "This is a variable font and SVG does not support OpenType Variations.\nPlease remove the SVG table from this font.",));
        }
        if !f.has_table(b"COLR") {
            problems.push(Status::fail(
                "add-colr",
                &format!(
                    "Font only has an SVG table. Please add a COLR table as well.\n{}",
                    NANOEMOJI_ADVICE,
                ),
            ));
        }
    }
    return_result(problems)
}
