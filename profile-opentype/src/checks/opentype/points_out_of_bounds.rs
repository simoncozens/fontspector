use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use read_fonts::{
    tables::glyf::{Glyph, PointFlags},
    types::Point,
    TableProvider,
};
use skrifa::GlyphId;

#[check(
    id = "opentype/points_out_of_bounds",
    rationale = "
        The glyf table specifies a bounding box for each glyph. This check
        ensures that all points in all glyph paths are within the bounding
        box. Glyphs with out-of-bounds points can cause rendering issues in
        some software, and should be corrected.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/735",
    title = "Check for points out of bounds"
)]
fn points_out_of_bounds(t: &Testable, context: &Context) -> CheckFnResult {
    let ttf = testfont!(t);
    let font = ttf.font();
    skip!(!ttf.has_table(b"glyf"), "no-glyf", "No glyf table");
    let glyf = font.glyf()?;
    let loca = font.loca(None)?;
    let mut messages = vec![];
    for gid in 0..font.maxp()?.num_glyphs() {
        let gid = GlyphId::new(gid.into());
        if let Some(Glyph::Simple(glyph)) = loca.get_glyf(gid, &glyf)? {
            let point_count = glyph.num_points();
            let mut points: Vec<Point<i32>> = vec![Point::default(); point_count];
            let mut flags = vec![PointFlags::default(); point_count];
            glyph.read_points_fast(&mut points, &mut flags)?;
            let x_min: i32 = glyph.x_min().into();
            let x_max: i32 = glyph.x_max().into();
            let y_min: i32 = glyph.y_min().into();
            let y_max: i32 = glyph.y_max().into();
            for point in points {
                if point.x < x_min || point.x > x_max {
                    messages.push(format!(
                        "{} (x={}, bounds are {}<->{})",
                        ttf.glyph_name_for_id_synthesise(gid),
                        point.x,
                        x_min,
                        x_max
                    ));
                }
                if point.y < y_min || point.y > y_max {
                    messages.push(format!(
                        "{} (y={}, bounds are {}<->{})",
                        ttf.glyph_name_for_id_synthesise(gid),
                        point.y,
                        y_min,
                        y_max
                    ));
                }
            }
        }
    }
    if messages.is_empty() {
        Ok(Status::just_one_pass())
    } else {
        Ok(Status::just_one_warn("points-out-of-bounds", 
        &format!("The following glyphs have coordinates which are out of bounds:\n\n{}\n\nThis happens a lot when points are not extremes, which is usually bad. However, fixing this alert by adding points on extremes may do more harm than good, especially with italics, calligraphic-script, handwriting, rounded and other fonts. So it is common to ignore this message.",
        bullet_list(context, messages)))
        )
    }
}
