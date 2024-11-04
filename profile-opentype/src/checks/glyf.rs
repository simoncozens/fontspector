use font_types::Point;
use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use read_fonts::{
    tables::glyf::{Anchor, Glyph, PointFlags},
    TableProvider,
};
use skrifa::{GlyphId, Tag};
use std::{cmp::Ordering, collections::HashSet};

#[check(
    id="opentype/glyf_unused_data",
    rationale="
        This check validates the structural integrity of the glyf table,
        by checking that all glyphs referenced in the loca table are
        actually present in the glyf table and that there is no unused
        data at the end of the glyf table. A failure here indicates a
        problem with the font compiler.
    ",
    proposal="https://github.com/fonttools/fontbakery/issues/4829",  // legacy check
    title="Is there any unused data at the end of the glyf table?"
)]
fn glyf_unused_data(t: &Testable, _context: &Context) -> CheckFnResult {
    let ttf = testfont!(t);
    let glyf = ttf
        .font()
        .table_data(Tag::new(b"glyf"))
        .ok_or(CheckError::skip("no-glyf", "No glyf table"))?;
    let loca = ttf
        .font()
        .loca(None)
        .map_err(|_| CheckError::Error("No loca table".to_string()))?;
    if let Some(last_index) = loca.get_raw(loca.len()) {
        Ok(match glyf.len().cmp(&(last_index as usize)) {
            Ordering::Greater => Status::just_one_fail(
                "unreachable-data",
                "Unused data at the end of the glyf table",
            ),
            Ordering::Less => {
                Status::just_one_fail("missing-data", "Missing data at the end of the glyf table")
            }
            Ordering::Equal => Status::just_one_pass(),
        })
    } else {
        Err(CheckError::Error("Invalid loca table".to_string()))
    }
}

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
fn check_point_out_of_bounds(t: &Testable, context: &Context) -> CheckFnResult {
    let ttf = testfont!(t);
    let font = ttf.font();
    skip!(!ttf.has_table(b"glyf"), "no-glyf", "No glyf table");
    let glyf = font.glyf()?;
    let loca = font.loca(None)?;
    let mut messages = vec![];
    for gid in 0..font.maxp()?.num_glyphs() {
        let gid = GlyphId::new(gid);
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

#[check(
    id = "opentype/glyf_non_transformed_duplicate_components",
    rationale = "
        There have been cases in which fonts had faulty double quote marks, with each
        of them containing two single quote marks as components with the same
        x, y coordinates which makes them visually look like single quote marks.

        This check ensures that glyphs do not contain duplicate components
        which have the same x,y coordinates.
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/2709",
    title = "Check glyphs do not have duplicate components which have the same x,y coordinates."
)]
fn check_glyf_non_transformed_duplicate_components(
    t: &Testable,
    context: &Context,
) -> CheckFnResult {
    let ttf = testfont!(t);
    let font = ttf.font();
    skip!(!ttf.has_table(b"glyf"), "no-glyf", "No glyf table");
    let glyf = font.glyf()?;
    let loca = font.loca(None)?;
    let mut messages = vec![];
    for gid in 0..font.maxp()?.num_glyphs() {
        let gid = GlyphId::new(gid);
        if let Some(Glyph::Composite(glyph)) = loca.get_glyf(gid, &glyf)? {
            let mut components = HashSet::new();
            for component in glyph.components() {
                if let Anchor::Offset { x, y } = component.anchor {
                    if !components.insert((component.glyph, x, y)) {
                        messages.push(format!(
                            "{}: duplicate component {} at {},{}",
                            ttf.glyph_name_for_id_synthesise(gid),
                            ttf.glyph_name_for_id_synthesise(component.glyph),
                            x,
                            y
                        ));
                    }
                }
            }
        }
    }
    if messages.is_empty() {
        Ok(Status::just_one_pass())
    } else {
        Ok(
            Status::just_one_warn("found-duplicates", 
                &format!("The following glyphs have duplicate components which have the same x,y coordinates.\n\n{}",
                    bullet_list(context, messages))
            )
        )
    }
}
