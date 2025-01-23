mod alignment_miss;
use fontspector_checkapi::{pens::BezGlyph, CheckError, TestFont, DEFAULT_LOCATION};
use skrifa::MetadataProvider;
use std::ops::Sub;

pub use alignment_miss::alignment_miss;
mod direction;
pub use direction::direction;
mod jaggy_segments;
pub use jaggy_segments::jaggy_segments;
mod semi_vertical;
pub use semi_vertical::semi_vertical;
mod short_segments;
pub use short_segments::short_segments;

pub(crate) fn close_but_not_on<T>(expected: T, actual: T, epsilon: T) -> bool
where
    T: Sub<Output = T> + PartialOrd + Copy + num_traits::sign::Signed,
{
    (actual - expected).abs() <= epsilon && actual != expected
}

pub(crate) fn name_and_bezglyph<'a>(
    f: &'a TestFont,
) -> impl Iterator<Item = (String, Result<BezGlyph, CheckError>)> + 'a {
    f.all_glyphs().map(|glyph| {
        let mut name = f.glyph_name_for_id_synthesise(glyph);
        if let Some((cp, _gid)) = f
            .font()
            .charmap()
            .mappings()
            .find(|(_cp, gid)| *gid == glyph)
        {
            name = format!("{} (U+{:04X})", name, cp);
        }
        let mut pen = BezGlyph::default();
        let result = f.draw_glyph(glyph, &mut pen, DEFAULT_LOCATION);
        (name, result.map(|_| pen))
    })
}
