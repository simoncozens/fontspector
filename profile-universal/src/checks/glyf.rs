use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::{
    tables::glyf::{Anchor, Glyph},
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
    if let Some(last_index) = loca.get_raw(loca.len() + 1) {
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
    let glyf = font.glyf()?;
    let loca = font.loca(None)?;
    let mut messages = vec![];
    for gid in 0..font.maxp()?.num_glyphs() {
        let gid = GlyphId::new(gid);
        if let Some(Glyph::Simple(glyph)) = loca.get_glyf(gid, &glyf)? {
            for point in glyph.points() {
                if point.x < glyph.x_min() || point.x > glyph.x_max() {
                    #[allow(clippy::unwrap_used)] // Synthesise is true so this will never fail
                    messages.push(format!(
                        "{} (x={}, bounds are {}<->{})",
                        ttf.glyph_name_for_id(gid, true).unwrap(),
                        point.x,
                        glyph.x_min(),
                        glyph.x_max()
                    ));
                }
                if point.y < glyph.y_min() || point.y > glyph.y_max() {
                    #[allow(clippy::unwrap_used)] // Synthesise is true so this will never fail
                    messages.push(format!(
                        "{} (y={}, bounds are {}<->{})",
                        ttf.glyph_name_for_id(gid, true).unwrap(),
                        point.y,
                        glyph.y_min(),
                        glyph.y_max()
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
                        #[allow(clippy::unwrap_used)] // Synthesise is true so this will never fail
                        messages.push(format!(
                            "{}: duplicate component {} at {},{}",
                            ttf.glyph_name_for_id(gid, true).unwrap(),
                            ttf.glyph_name_for_id(component.glyph, true).unwrap(),
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
