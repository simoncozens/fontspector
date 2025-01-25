use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;
use skrifa::{GlyphId, MetadataProvider};


const AVG_CHAR_WEIGHTS: [(char, u32); 27] = [
    ('a', 64),
    ('b', 14),
    ('c', 27),
    ('d', 35),
    ('e', 100),
    ('f', 20),
    ('g', 14),
    ('h', 42),
    ('i', 63),
    ('j', 3),
    ('k', 6),
    ('l', 35),
    ('m', 20),
    ('n', 56),
    ('o', 56),
    ('p', 17),
    ('q', 4),
    ('r', 49),
    ('s', 56),
    ('t', 71),
    ('u', 31),
    ('v', 10),
    ('w', 18),
    ('x', 3),
    ('y', 18),
    ('z', 2),
    (' ', 166),
];

#[check(
    id = "opentype/xavgcharwidth",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Checking OS/2 fsSelection value.",
    rationale = "
        The OS/2.xAvgCharWidth field is used to calculate the width of a string of
        characters. It is the average width of all non-zero width glyphs in the font.

        This check ensures that the value is correct. A failure here may indicate
        a bug in the font compiler, rather than something that the designer can
        do anything about.
    "
)]
fn xavgcharwidth(f: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    let os2 = font.font().os2()?;
    let hmtx = font.font().hmtx()?;
    let charmap = font.font().charmap();
    let (rule, expected) = if os2.version() >= 3 {
        let advances = hmtx
            .h_metrics()
            .iter()
            .map(|metric| metric.advance.get() as u32)
            .filter(|&w| w > 0)
            .collect::<Vec<_>>();
        if advances.is_empty() {
            return Err(CheckError::Error(
                "No non-zero width glyphs in font for average character width calculation"
                    .to_string(),
            ));
        }
        (
            "the average of the widths of all glyphs in the font",
            advances.iter().sum::<u32>() / advances.len() as u32,
        )
    } else {
        let ids: Vec<Option<GlyphId>> = AVG_CHAR_WEIGHTS
            .iter()
            .map(|(c, _)| charmap.map(*c))
            .collect();
        if ids.iter().any(|id| id.is_none()) {
            return Err(CheckError::Error(
                "Missing glyph in font for average character width calculation".to_string(),
            ));
        }
        #[allow(clippy::unwrap_used)] // We know all the characters are in the font
        let advances = ids
            .iter()
            .zip(AVG_CHAR_WEIGHTS.iter())
            .map(|(id, (_, w))| hmtx.advance(id.unwrap()).unwrap_or(0) as u32 * w)
            .collect::<Vec<_>>();
        (
            "the weighted average of the widths of the latin lowercase glyphs in the font",
            advances.iter().sum::<u32>() / 1000u32,
        )
    };
    let actual = os2.x_avg_char_width();
    let difference = (expected as i16).abs_diff(actual);
    Ok(match difference {
        0 => Status::just_one_pass(),
        1..=10 => Status::just_one_info(
            "xAvgCharWidth-close",
            &format!("OS/2 xAvgCharWidth is {} but it should be {} which corresponds to {}. These are similar values, which may be a symptom of the slightly different calculation of the xAvgCharWidth value in font editors. There's further discussion on this at https://github.com/fonttools/fontbakery/issues/1622",
                actual, expected, rule
            )
        ),
        _ => Status::just_one_warn(
            "xAvgCharWidth-wrong",
            &format!("OS/2 xAvgCharWidth is {} but it should be {} which corresponds to {}. This may indicate a problem with the font editor or the font compiler.",
                actual, expected, rule
            )
        )
    })
}
