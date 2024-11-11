use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use interpolatable::{run_tests, Problem, ProblemDetails};
use read_fonts::{tables::fvar::VariationAxisRecord, ReadError, TableProvider};
use skrifa::{setting::VariationSetting, FontRef, GlyphId};

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
fn poor_mans_denormalize(peak: f32, axis: &VariationAxisRecord) -> f32 {
    // Insert avar here
    if peak > 0.0 {
        lerp(
            axis.default_value().to_f32(),
            axis.max_value().to_f32(),
            peak,
        )
    } else {
        lerp(
            axis.default_value().to_f32(),
            axis.min_value().to_f32(),
            -peak,
        )
    }
}

pub trait DenormalizeLocation {
    /// Given a normalized location tuple, turn it back into a friendly representation in userspace
    fn denormalize_location(&self, tuple: &[f32]) -> Result<Vec<VariationSetting>, ReadError>;
}

impl DenormalizeLocation for FontRef<'_> {
    fn denormalize_location(&self, tuple: &[f32]) -> Result<Vec<VariationSetting>, ReadError> {
        let all_axes = self.fvar()?.axes()?;
        Ok(all_axes
            .iter()
            .zip(tuple)
            .filter(|&(_axis, peak)| *peak != 0.0)
            .map(|(axis, peak)| {
                let value = poor_mans_denormalize(*peak, axis);
                (axis.axis_tag().to_string().as_str(), value).into()
            })
            .collect())
    }
}

fn glyph_variations(font: &FontRef, gid: GlyphId) -> Result<Vec<Vec<VariationSetting>>, ReadError> {
    font.gvar()?.glyph_variation_data(gid).map(|data| {
        data.tuples()
            .flat_map(|t| {
                let tuple: Vec<f32> = t.peak().values.iter().map(|v| v.get().to_f32()).collect();
                font.denormalize_location(&tuple)
            })
            .collect()
    })
}

fn problem_report(p: &Problem) -> String {
    match &p.details {
        ProblemDetails::PathCount { count_1, count_2 } => {
            format!(
                "Path count mismatch: {} in {} vs {} in {}",
                count_1, p.master_1_name, count_2, p.master_2_name
            )
        }
        ProblemDetails::NodeCount { count_1, count_2 } => {
            format!(
                "Node count mismatch: {} in {} vs {} in {}",
                count_1, p.master_1_name, count_2, p.master_2_name
            )
        }
        ProblemDetails::NodeIncompatibility {
            is_control_1,
            is_control_2,
        } => format!(
            "Incompatible nodes: mismatch: {} is {} in {} vs {} in {}",
            p.node.unwrap_or(0),
            (if *is_control_1 {
                "off-curve"
            } else {
                "on-curve"
            }),
            p.master_1_name,
            (if *is_control_2 {
                "off-curve"
            } else {
                "on-curve"
            }),
            p.master_2_name,
        ),
        ProblemDetails::ContourOrder { order_1, order_2 } => format!(
            "Contour order mismatch: {:?} in {} vs {:?} in {}",
            order_1, p.master_1_name, order_2, p.master_2_name
        ),
        ProblemDetails::WrongStartPoint {
            proposed_point,
            reverse,
        } => format!(
            "Wrong start point: contour {} should start at {} in {}{}",
            p.contour.unwrap_or(0),
            proposed_point,
            p.master_2_name,
            if *reverse {
                " (and contour should be reversed)"
            } else {
                ""
            }
        ),
        ProblemDetails::Overweight {
            value_1: _,
            value_2: _,
        } => {
            format!(
                "Contour {} becomes overweight in {} compared to {}",
                p.contour.unwrap_or(0),
                p.master_2_name,
                p.master_1_name
            )
        }
        ProblemDetails::Underweight {
            value_1: _,
            value_2: _,
        } => {
            format!(
                "Contour {} becomes underweight in {} compared to {}",
                p.contour.unwrap_or(0),
                p.master_2_name,
                p.master_1_name
            )
        }
        ProblemDetails::Kink => format!(
            "Kink in contour {} at node {}",
            p.contour.unwrap_or(0),
            p.node.unwrap_or(0)
        ),
    }
}
#[check(
    id = "interpolation_issues",
    rationale = "
        When creating a variable font, the designer must make sure that corresponding
        paths have the same start points across masters, as well as that corresponding
        component shapes are placed in the same order within a glyph across masters.
        If this is not done, the glyph will not interpolate correctly.

        Here we check for the presence of potential interpolation errors using the
        interpolatable crate.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3930",
    title = "Detect any interpolation issues in the font."
)]
fn interpolation_issues(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let font = f.font();
    let upem = font.head()?.units_per_em();
    skip!(
        !f.is_variable_font(),
        "not-variable",
        "Font is not variable"
    );

    let mut result: Vec<_> = vec![];
    let mut locations: Vec<Vec<VariationSetting>> = vec![vec![]];
    for gid in f.all_glyphs() {
        let glyphname = f.glyph_name_for_id_synthesise(gid);
        let mut default_glyph = interpolatable::Glyph::new_from_font(&font, gid, &[]).ok_or(
            CheckError::Error(format!("Can't convert glyph {}", glyphname)),
        )?;
        default_glyph.master_name = "default".to_string();
        default_glyph.master_index = 0;
        if let Ok(variations) = glyph_variations(&font, gid) {
            for variation in variations {
                let mut glyph =
                    interpolatable::Glyph::new_from_font(&font, gid, &variation).ok_or(
                        CheckError::Error(format!("Can't convert glyph {}", glyphname)),
                    )?;
                glyph.master_name = variation
                    .iter()
                    .map(|v| format!("{}={}", v.selector, v.value))
                    .collect::<Vec<_>>()
                    .join(",");
                if !locations.contains(&variation) {
                    locations.push(variation.clone());
                }
                glyph.master_index = locations
                    .iter()
                    .position(|x| x == &variation)
                    .ok_or(CheckError::Error("Can't find master index".to_string()))?;
                let problems = run_tests(&default_glyph, &glyph, None, None, Some(upem));
                if !problems.is_empty() {
                    result.push(Status::warn(
                        "glyph",
                        &format!(
                            "Glyph {} has interpolation issues:\n{}",
                            glyphname,
                            bullet_list(
                                context,
                                problems.iter().map(problem_report).collect::<Vec<_>>()
                            )
                        ),
                    ))
                }
            }
        }
    }
    return_result(result)
}
