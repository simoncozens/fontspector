use fontspector_checkapi::{fixfont, prelude::*, testfont, FileTypeConvert};
use hashbrown::HashMap;
use itertools::Itertools;
use kurbo::Affine;
use read_fonts::{
    tables::glyf::{Anchor, CurvePoint, Glyph, Transform},
    types::F2Dot14,
    FontData, TableProvider,
};
use skrifa::GlyphId;
use write_fonts::{
    from_obj::ToOwnedObj,
    tables::glyf::{
        Component, CompositeGlyph, Contour, GlyfLocaBuilder, Glyph as WriteGlyph, SimpleGlyph,
    },
    FontBuilder,
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
    title = "Ensure component transforms do not perform scaling or rotation.",
    hotfix = decompose_transformed_components
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

fn decompose_transformed_components(t: &Testable) -> FixFnResult {
    let f = fixfont!(t);
    let loca = f
        .font()
        .loca(None)
        .map_err(|_| "loca table not found".to_string())?;
    let glyf = f
        .font()
        .glyf()
        .map_err(|_| "glyf table not found".to_string())?;
    let bad_composites = f
        .all_glyphs()
        .filter_map(|gid| {
            loca.get_glyf(gid, &glyf)
                .ok()
                .flatten()
                .and_then(|glyph| match glyph {
                    Glyph::Composite(composite)
                        if composite.components().any(|component| {
                            !transform_is_linear(component.transform)
                                || transform_is_semi_flipped(component.transform)
                        }) =>
                    {
                        Some(gid)
                    }
                    _ => None,
                })
        })
        .collect::<Vec<_>>();

    decompose_components_impl(t, &bad_composites)
}

pub(crate) fn decompose_components_impl(t: &Testable, decompose_order: &[GlyphId]) -> FixFnResult {
    let f = fixfont!(t);
    let mut new_font = FontBuilder::new();
    let mut builder = GlyfLocaBuilder::new();
    let loca = f
        .font()
        .loca(None)
        .map_err(|_| "loca table not found".to_string())?;
    let glyf = f
        .font()
        .glyf()
        .map_err(|_| "glyf table not found".to_string())?;
    let mut all_glyphs: HashMap<GlyphId, WriteGlyph> = f
        .all_glyphs()
        .map(|gid| {
            loca.get_glyf(gid, &glyf)
                .map(|option_glyph| {
                    option_glyph
                        .map(|glyph| {
                            let g: WriteGlyph = glyph.to_owned_obj(FontData::new(&[]));
                            g
                        })
                        .unwrap_or(WriteGlyph::Empty)
                })
                .map(|x| (gid, x))
        })
        .collect::<Result<HashMap<GlyphId, WriteGlyph>, _>>()
        .map_err(|x| x.to_string())?;
    for glyph_id in decompose_order {
        let current_glyph = all_glyphs.get(glyph_id).ok_or("glyph not found")?;
        match current_glyph {
            WriteGlyph::Composite(composite) => {
                let new_glyph = decompose_glyph(composite, &all_glyphs)?;
                all_glyphs.insert(*glyph_id, new_glyph);
            }
            WriteGlyph::Empty | WriteGlyph::Simple(_) => {}
        }
    }
    for glyph_id in all_glyphs.keys().sorted_by(|a, b| a.cmp(b)) {
        let glyph = all_glyphs.get(glyph_id).ok_or("glyph not found")?;
        builder.add_glyph(glyph).map_err(|x| x.to_string())?;
    }
    let (new_glyph, new_loca, _head_format) = builder.build();
    new_font.add_table(&new_glyph).map_err(|x| x.to_string())?;
    new_font.add_table(&new_loca).map_err(|x| x.to_string())?;
    new_font.copy_missing_tables(f.font());
    let new_bytes = new_font.build();
    std::fs::write(&t.filename, new_bytes).map_err(|_| "Couldn't write file".to_string())?;

    Ok(true)
}

fn decompose_glyph(
    composite: &CompositeGlyph,
    glyphs: &HashMap<GlyphId, WriteGlyph>,
) -> Result<WriteGlyph, String> {
    let mut new_glyph = SimpleGlyph::default();
    for component in composite.components() {
        for (gid, affine) in flatten_component(glyphs, component)? {
            let component_glyph = glyphs.get(&gid).ok_or("glyph not found")?;
            match component_glyph {
                WriteGlyph::Simple(simple) => {
                    new_glyph
                        .contours
                        .extend(simple.contours.iter().map(|c| transform_contour(c, affine)));
                }
                _ => {
                    panic!("unexpected glyph type")
                }
            }
        }
    }
    Ok(WriteGlyph::Simple(new_glyph))
}

fn transform_contour(c: &Contour, affine: Affine) -> Contour {
    c.iter()
        .map(|point| {
            let kurbo_pt = kurbo::Point::new(point.x as f64, point.y as f64);
            let new_pt = affine * kurbo_pt;
            CurvePoint::new(new_pt.x as i16, new_pt.y as i16, point.on_curve)
        })
        .collect::<Vec<_>>()
        .into()
}

fn flatten_component(
    glyphs: &HashMap<GlyphId, WriteGlyph>,
    component: &Component,
) -> Result<Vec<(GlyphId, kurbo::Affine)>, String> {
    let glyph = glyphs
        .get(&GlyphId::from(component.glyph))
        .ok_or("glyph not found")?;
    let my_transform = to_kurbo_transform(&component.transform, &component.anchor);

    Ok(match glyph {
        WriteGlyph::Empty => vec![],
        WriteGlyph::Simple(_) => {
            vec![(component.glyph.into(), my_transform)]
        }
        WriteGlyph::Composite(composite_glyph) => {
            let mut all_flattened_components = vec![];
            for component in composite_glyph.components() {
                all_flattened_components.extend(flatten_component(glyphs, component)?.iter().map(
                    |(gid, transform)| {
                        let new_transform = my_transform * *transform;
                        (*gid, new_transform)
                    },
                ));
            }
            all_flattened_components
        }
    })
}

fn to_kurbo_transform(transform: &Transform, anchor: &Anchor) -> kurbo::Affine {
    let (dx, dy) = match anchor {
        Anchor::Offset { x, y } => (*x, *y),
        Anchor::Point { .. } => (0, 0),
    };
    kurbo::Affine::new([
        transform.xx.to_f32() as f64,
        transform.xy.to_f32() as f64, // check
        transform.yx.to_f32() as f64, // check
        transform.yy.to_f32() as f64,
        dx as f64,
        dy as f64,
    ])
}
