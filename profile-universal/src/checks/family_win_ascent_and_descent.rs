use fontspector_checkapi::{prelude::*, FileTypeConvert, TestFont};
use read_fonts::{
    tables::glyf::{Glyph, SimpleGlyph},
    TableProvider,
};

#[derive(Debug, Default)]
struct Metrics {
    ymin: i16,
    ymax: i16,
}

#[allow(dead_code)]
fn all_simple_glyphs<'a>(
    font: &'a TestFont,
) -> Result<impl Iterator<Item = SimpleGlyph<'a>>, CheckError> {
    let loca = font
        .font()
        .loca(None)
        .map_err(|_| CheckError::Error("loca table not found".to_string()))?;
    let glyf = font
        .font()
        .glyf()
        .map_err(|_| CheckError::Error("glyf table not found".to_string()))?;
    Ok(font.all_glyphs().filter_map(move |glyphid| {
        if let Some(Glyph::Simple(simple)) = loca.get_glyf(glyphid, &glyf).ok()? {
            Some(simple)
        } else {
            None
        }
    }))
}

fn family_metrics(fonts: &[TestFont]) -> Result<Metrics, CheckError> {
    let mut metrics = Metrics::default();
    for font in fonts {
        // The original fontbakery code checked whether the font was
        // OpenType or TrueType, and if TrueType went with max/min values
        // of all the glyphs in the glyf table. But the "OpenType or TrueType"
        // test was broken, and it seems like it used the head table values
        // for "OpenType" fonts anyway. So we'll just use the head table values.
        // If you want the other code later, here it is:
        /*
        let (font_ymin, font_ymax) =
        if font.has_table(b"glyf") {
            (
                all_simple_glyphs(font)?
                    .map(|g| g.y_min())
                    .min()
                    .unwrap_or(0),
                all_simple_glyphs(font)?
                    .map(|g| g.y_max())
                    .max()
                    .unwrap_or(0),
            )
        } else
         */
        let (font_ymin, font_ymax) = {
            let head = font.font().head()?;
            (head.y_min(), head.y_max())
        };
        metrics.ymin = metrics.ymin.min(font_ymin);
        metrics.ymax = metrics.ymax.max(font_ymax);
    }
    Ok(metrics)
}

#[check(
    id = "family/win_ascent_and_descent",
    rationale = "
        A font's winAscent and winDescent values should be greater than or equal to
        the head table's yMax, abs(yMin) values. If they are less than these values,
        clipping can occur on Windows platforms
        (https://github.com/RedHatBrand/Overpass/issues/33).

        If the font includes tall/deep writing systems such as Arabic or Devanagari,
        the winAscent and winDescent can be greater than the yMax and absolute yMin
        values to accommodate vowel marks.

        When the 'win' Metrics are significantly greater than the UPM, the linespacing
        can appear too loose. To counteract this, enabling the OS/2 fsSelection
        bit 7 (Use_Typo_Metrics), will force Windows to use the OS/2 'typo' values
        instead. This means the font developer can control the linespacing with
        the 'typo' values, whilst avoiding clipping by setting the 'win' values to
        values greater than the yMax and absolute yMin.
    ",
    implementation = "all",
    title = "Checking OS/2 usWinAscent & usWinDescent",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829"
)]
fn family_win_ascent_and_descent(c: &TestableCollection, _context: &Context) -> CheckFnResult {
    let fonts = TTF.from_collection(c);
    let metrics = family_metrics(&fonts)?;
    let mut problems = vec![];
    for font in fonts.iter() {
        let os2 = font.font().os2()?;
        if (os2.us_win_ascent() as i16) < metrics.ymax {
            problems.push(Status::fail(
                "ascent",
                &format!("OS/2.usWinAscent value should be equal or greater than {}, but got {} instead.",
                            metrics.ymax, os2.us_win_ascent()),
            ));
        }
        if (os2.us_win_ascent() as i16) > 2 * metrics.ymax {
            problems.push(Status::fail(
                "ascent",
                &format!("OS/2.usWinAscent value {} is too large. It should be less than double the yMax. Current yMax value is {}.",
                os2.us_win_ascent(), metrics.ymax)
            ));
        }
        if (os2.us_win_descent() as i16) < metrics.ymin.abs() {
            problems.push(Status::fail(
                "descent",
                &format!("OS/2.usWinDescent value should be equal or greater than {}, but got {} instead.",
                            metrics.ymin.abs(), os2.us_win_descent()),
            ));
        }
        if (os2.us_win_descent() as i16) > 2 * metrics.ymin.abs() {
            problems.push(Status::fail(
                "descent",
                &format!("OS/2.usWinDescent value {} is too large. It should be less than double the yMin. Current absolute yMin value is {}.",
                os2.us_win_descent(), metrics.ymin.abs())
            ));
        }
    }
    return_result(problems)
}
