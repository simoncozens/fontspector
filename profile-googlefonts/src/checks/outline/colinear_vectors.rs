use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use itertools::Itertools;

use crate::checks::outline::name_and_bezglyph;

const COLINEAR_EPSILON: f64 = 0.1; // Radians

#[check(
    id = "outline_colinear_vectors",
    rationale = "
        
        This check looks for consecutive line segments which have the same angle. This
        normally happens if an outline point has been added by accident.

        This check is not run for variable fonts, as they may legitimately have
        colinear vectors.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/3088",
    title = "Do any segments have colinear vectors?"
)]
fn colinear_vectors(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut all_warnings = vec![];
    skip!(
        f.is_variable_font(),
        "variable-font",
        "This check produces too many false positives with variable fonts."
    );
    for (name, result) in name_and_bezglyph(&f) {
        let pen = result?;
        for contour in pen.iter() {
            let segs = contour.segments().collect::<Vec<_>>();
            for (prev, next) in segs.iter().circular_tuple_windows() {
                if let (kurbo::PathSeg::Line(prev), kurbo::PathSeg::Line(next)) = (prev, next) {
                    let prev_angle = (prev.p1 - prev.p0).angle();
                    let next_angle = (next.p1 - next.p0).angle();
                    if (prev_angle - next_angle).abs() < COLINEAR_EPSILON {
                        all_warnings.push(format!("{}: {:?} -> {:?}", name, prev, next));
                    }
                }
            }
            if all_warnings.len() > 100 {
                return Ok(Status::just_one_pass());
            }
        }
    }
    Ok(if !all_warnings.is_empty() {
        Status::just_one_warn(
            "found-colinear-vectors",
            &format!(
                "The following glyphs have colinear vectors:\n\n{}",
                bullet_list(context, all_warnings)
            ),
        )
    } else {
        Status::just_one_pass()
    })
}
