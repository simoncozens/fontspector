use std::collections::HashSet;

use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use kurbo::ParamCurve;

use crate::checks::outline::name_and_bezglyph;

#[check(
    id = "overlapping_path_segments",
    rationale = "
        
        Some rasterizers encounter difficulties when rendering glyphs with
        overlapping path segments.

        A path segment is a section of a path defined by two on-curve points.
        When two segments share the same coordinates, they are considered
        overlapping.
    
    ",
    proposal = "https://github.com/google/fonts/issues/7594#issuecomment-2401909084",
    title = "Check there are no overlapping path segments"
)]
fn overlapping_path_segments(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut all_warnings = vec![];
    for (name, result) in name_and_bezglyph(&f) {
        let mut seen = HashSet::new();
        let pen = result?;
        for contour in pen.iter() {
            for seg in contour.segments() {
                // Urgh, we can't compare Points, so we have to approximate by stringifying
                let normal = format!("{}/{}", seg.eval(0.0), seg.eval(1.0));
                let flipped = format!("{}/{}", seg.eval(1.0), seg.eval(0.0));
                if seen.contains(&normal) || seen.contains(&flipped) {
                    all_warnings.push(format!(
                        "{}: {:?} has the same coordinates as a previous segment.",
                        name, seg
                    ));
                }
                seen.insert(normal);
            }
        }
    }
    Ok(if !all_warnings.is_empty() {
        Status::just_one_warn(
            "overlapping-path-segments",
            &format!(
                "The following glyphs have overlapping path segments:\n\n{}",
                bullet_list(context, all_warnings)
            ),
        )
    } else {
        Status::just_one_pass()
    })
}
