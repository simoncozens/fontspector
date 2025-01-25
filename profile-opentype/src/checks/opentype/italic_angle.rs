use fontspector_checkapi::{
    pens::BezGlyph, prelude::*, testfont, FileTypeConvert, DEFAULT_LOCATION,
};
use kurbo::{BezPath, ParamCurve};
use read_fonts::{types::BoundingBox, TableProvider};
use skrifa::{
    prelude::{LocationRef, Size},
    MetadataProvider,
};

fn x_leftmost_intersection(paths: &[BezPath], y: f32, x_min: f32, x_max: f32) -> Option<f32> {
    let mut y_adjust = 0.0;
    while y_adjust < 20.0 {
        let line = kurbo::Line::new((x_min - 100.0, y + y_adjust), (x_max + 100.0, y + y_adjust));
        for path in paths {
            for seg in path.segments() {
                if let Some(intersection) = seg.intersect_line(line).first() {
                    let point = line.eval(intersection.line_t);
                    return Some(point.x as f32);
                }
            }
        }
        y_adjust += 2.0;
    }
    None
}
#[check(
    id="opentype/italic_angle",
    rationale="
        The 'post' table italicAngle property should be a reasonable amount, likely
        not more than 30°. Note that in the OpenType specification, the value is
        negative for a rightward lean.

        https://docs.microsoft.com/en-us/typography/opentype/spec/post
    ",
    proposal="https://github.com/fonttools/fontbakery/issues/4829",  // legacy check
    title="Checking post.italicAngle value."
)]
fn italic_angle(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let value = f.font().post()?.italic_angle().to_f32();
    let mut problems = vec![];
    let to_test: [u32; 4] = [0x7c, 0x5b, 0x48, 0x49];
    let gids = to_test.iter().flat_map(|&cp| f.font().charmap().map(cp));
    let metrics = f
        .font()
        .glyph_metrics(Size::unscaled(), LocationRef::new(&[]));

    let mut calculated_italic_angle = None;
    let empty = BoundingBox {
        x_min: 0.0,
        y_min: 0.0,
        x_max: 0.0,
        y_max: 0.0,
    };
    let bad_gids: Vec<_> = gids
        .clone()
        .filter(|gid| metrics.bounds(*gid) == Some(empty))
        .collect();
    for gid in gids {
        if let Some(bounds) = metrics.bounds(gid) {
            if bounds == empty {
                continue;
            }
            let mut paths = BezGlyph::default();
            f.draw_glyph(gid, &mut paths, DEFAULT_LOCATION)?;
            let y_bottom = bounds.y_min + (bounds.y_max - bounds.y_min) * 0.2;
            let y_top = bounds.y_min + (bounds.y_max - bounds.y_min) * 0.8;
            if let Some(x_intsctn_bottom) =
                x_leftmost_intersection(&paths.0, y_bottom, bounds.x_min, bounds.x_max)
            {
                if let Some(x_intsctn_top) =
                    x_leftmost_intersection(&paths.0, y_top, bounds.x_min, bounds.x_max)
                {
                    let x_d = x_intsctn_top - x_intsctn_bottom;
                    let y_d = y_top - y_bottom;
                    calculated_italic_angle = Some(-(x_d.atan2(y_d).to_degrees()));
                    break;
                }
            }
        }
    }
    if let Some(calculated_italic_angle) = calculated_italic_angle {
        if calculated_italic_angle < 0.1 && value > 0.0 {
            problems.push(Status::warn("positive",
                &format!("The value of post.italicAngle is positive, which is likely a mistake and should become negative for right-leaning Italics.\npost.italicAngle: {}\nangle calculated from outlines: {:.1})",value, calculated_italic_angle)));
        }
        if calculated_italic_angle > 0.1 && value < 0.0 {
            problems.push(Status::warn("negative",
                &format!("The value of post.italicAngle is negative, which is likely a mistake and should become positive for left-leaning Italics.\npost.italicAngle: {}\nangle calculated from outlines: {:.1})",value, calculated_italic_angle)));
        }
    } else {
        // Fall back to checking it's positive
        if value > 0.0 {
            problems.push(Status::warn("positive", "The value of post.italicAngle is positive, which is likely a mistake and should become negative for right-leaning Italics. If this font is left-leaning, ignore this warning"))
        }
    }

    if value.abs() > 90.0 {
        problems.push(Status::fail(
            "over-90-degrees",
            "The value of post.italicAngle is over 90°, which is surely a mistake.",
        ))
    }
    if value.abs() > 30.0 {
        problems.push(Status::warn(
            "over-30-degrees",
            &format!("The value of post.italicAngle ({}) is very high (over -30° or 30°, and should be confirmed.", value),
        ))
    } else if value.abs() > 20.0 {
        problems.push(Status::warn(
            "over-20-degrees",
            &format!("The value of post.italicAngle ({}) seem very high (over -20° or 20°, and should be confirmed.", value),
        ))
    }
    if let Some(style) = f.style() {
        if style.contains("Italic") {
            if value == 0.0 {
                problems.push(Status::fail(
                    "zero-italic",
                    "Font is italic, so post.italicAngle should be non-zero.",
                ))
            }
        } else if value != 0.0 {
            problems.push(Status::fail(
                "non-zero-upright",
                "Font is not italic, so post.italicAngle should equal to zero.",
            ))
        }
    }
    if !bad_gids.is_empty() {
        problems.push(Status::warn(
            "empty-glyphs",
            &format!(
                "The following glyphs were present but did not contain any outlines:\n{}",
                bullet_list(
                    context,
                    bad_gids
                        .iter()
                        .map(|&gid| f.glyph_name_for_id_synthesise(gid))
                )
            ),
        ))
    }
    return_result(problems)
}
