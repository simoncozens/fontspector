use fontspector_checkapi::{pens::BezGlyph, CheckError, TestFont, DEFAULT_LOCATION};
use skrifa::MetadataProvider;
use std::ops::Sub;

mod alignment_miss;
mod colinear_vectors;
mod direction;
mod jaggy_segments;
mod overlapping_path_segments;
mod semi_vertical;
mod short_segments;
pub use alignment_miss::alignment_miss;
pub use colinear_vectors::colinear_vectors;
pub use direction::direction;
pub use jaggy_segments::jaggy_segments;
pub use overlapping_path_segments::overlapping_path_segments;
pub use semi_vertical::semi_vertical;
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
