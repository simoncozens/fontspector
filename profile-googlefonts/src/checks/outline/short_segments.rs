use crate::checks::outline::name_and_bezglyph;
use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use kurbo::{ParamCurveArclen, PathSeg, Shape};

const SHORT_PATH_ABSOLUTE_EPSILON: f64 = 3.0;
const SHORT_PATH_RELATIVE_EPSILON: f64 = 0.006;

fn segment_is_short(pathseg: &PathSeg, total_length: f64, prev_was_line: bool) -> bool {
    let len = pathseg.arclen(0.01);
    // An *very* short segment is likely to be a mistake
    if len <= 1.0e-9 {
        return true;
    }
    let short_seg =
        len < SHORT_PATH_ABSOLUTE_EPSILON || len < SHORT_PATH_RELATIVE_EPSILON * total_length;
    let current_is_curve = matches!(pathseg, PathSeg::Cubic(_) | PathSeg::Quad(_));

    short_seg && (prev_was_line || current_is_curve)
}

#[check(
    id = "outline_short_segments",
    rationale = "
        
        This check looks for outline segments which seem particularly short (less
        than 0.6% of the overall path length).

        This check is not run for variable fonts, as they may legitimately have
        short segments. As this check is liable to generate significant numbers
        of false positives, it will pass if there are more than
        100 reported short segments.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/3088",
    title = "Are any segments inordinately short?"
)]
fn short_segments(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut all_warnings = vec![];
    skip!(
        f.is_variable_font(),
        "variable-font",
        "This check produces too many false positives with variable fonts."
    );
    for (name, result) in name_and_bezglyph(&f) {
        let pen = result?;
        for path in pen.iter() {
            let outline_length = path.perimeter(0.01);
            let segments = path.segments().collect::<Vec<_>>();
            if segments.is_empty() {
                continue;
            }
            #[allow(clippy::unwrap_used)] // We just checked it has a segment
            let mut prev_was_line = matches!(segments.last().unwrap(), kurbo::PathSeg::Line(_));
            for seg in segments.iter() {
                if segment_is_short(seg, outline_length, prev_was_line) {
                    all_warnings.push(format!("{} contains a short segment {:?}", name, seg));
                }
                prev_was_line = matches!(seg, kurbo::PathSeg::Line(_));
            }
            if all_warnings.len() > 100 {
                return Ok(Status::just_one_pass());
            }
        }
    }
    Ok(if !all_warnings.is_empty() {
        Status::just_one_warn(
            "found-short-segments",
            &format!(
                "The following glyphs have short segments:\n\n{}",
                bullet_list(context, all_warnings)
            ),
        )
    } else {
        Status::just_one_pass()
    })
}
