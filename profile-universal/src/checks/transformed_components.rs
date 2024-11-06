use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::{
    tables::glyf::{Glyph, Transform},
    types::F2Dot14,
    TableProvider,
};

fn transform_is_linear(t: Transform) -> bool {
    t.xx == F2Dot14::from_f32(1.0)
        && t.xy == F2Dot14::from_f32(0.0)
        && t.yx == F2Dot14::from_f32(0.0)
        && t.yy == F2Dot14::from_f32(1.0)
}

fn transform_is_semi_flipped(t: Transform) -> bool {
    t.xx.to_f32() * t.yy.to_f32() < 0.0
}

#[check(
    id = "transformed_components",
    rationale = "
        Some families have glyphs which have been constructed by using
        transformed components e.g the 'u' being constructed from a flipped 'n'.

        From a designers point of view, this sounds like a win (less work).
        However, such approaches can lead to rasterization issues, such as
        having the 'u' not sitting on the baseline at certain sizes after
        running the font through ttfautohint.

        Other issues are outlines that end up reversed when only one dimension
        is flipped while the other isn't.

        As of July 2019, Marc Foley observed that ttfautohint assigns cvt values
        to transformed glyphs as if they are not transformed and the result is
        they render very badly, and that vttLib does not support flipped components.

        When building the font with fontmake, the problem can be fixed by adding
        this to the command line:

        --filter DecomposeTransformedComponentsFilter
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/2011",
    title = "Ensure component transforms do not perform scaling or rotation."
)]
fn transformed_components(f: &Testable, context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    let loca = font
        .font()
        .loca(None)
        .map_err(|_| CheckError::skip("no-loca", "loca table not found"))?;
    let glyf = font
        .font()
        .glyf()
        .map_err(|_| CheckError::skip("no-glyf", "glyf table not found"))?;
    let is_hinted = font.has_table(b"fpgm");
    let mut failures = vec![];
    for glyphid in font.all_glyphs() {
        if let Some(glyph) = loca.get_glyf(glyphid, &glyf)? {
            match glyph {
                Glyph::Simple(_) => {}
                Glyph::Composite(composite) => {
                    for component in composite.components() {
                        if is_hinted {
                            if !transform_is_linear(component.transform) {
                                let glyph_name = font.glyph_name_for_id_synthesise(glyphid);
                                let component_name =
                                    font.glyph_name_for_id_synthesise(component.glyph);
                                failures.push(format!("{glyph_name} (component {component_name})"));
                            }
                        } else if transform_is_semi_flipped(component.transform) {
                            let glyph_name = font.glyph_name_for_id_synthesise(glyphid);
                            let component_name = font.glyph_name_for_id_synthesise(component.glyph);
                            failures.push(format!("{glyph_name} (component {component_name})"));
                        }
                    }
                }
            }
        }
    }
    if failures.is_empty() {
        Ok(Status::just_one_pass())
    } else {
        Ok(Status::just_one_fail(
            "transformed-components",
            &format!("The following glyphs had components with scaling or rotation or inverted outline direction:\n{}",
            bullet_list(context, failures))
        ))
    }
}
