use std::collections::{HashMap, HashSet};

use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use itertools::Itertools;
use read_fonts::{
    tables::{
        glyf::Glyph,
        gpos::{
            DeviceOrVariationIndex::{Device, VariationIndex},
            PairPosFormat1, PairPosFormat2,
        },
        variations::ItemVariationStore,
    },
    FontData, ReadError, TableProvider,
};
use skrifa::{GlyphId, MetadataProvider};

#[check(
    id = "varfont/duplexed_axis_reflow",
    rationale = "
        
        Certain axes, such as grade (GRAD) or roundness (ROND), should not
        change any advanceWidth or kerning data across the font's design space.
        This is because altering the advance width of glyphs can cause text reflow.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3187",
    title = "Ensure VFs with duplexed axes do not vary horizontal advance."
)]
fn varfont_duplexed_axis_reflow(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    const DUPLEXED_AXES: [&str; 2] = ["GRAD", "ROND"];
    let axis_indices = f
        .font()
        .axes()
        .iter()
        .enumerate()
        .filter(|(_, axis)| {
            DUPLEXED_AXES
                .iter()
                .any(|&duplexed_axis| axis.tag() == duplexed_axis)
        })
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    skip!(
        axis_indices.is_empty(),
        "no-relevant-axes",
        "This font has no duplexed axes"
    );

    let gvar = f.font().gvar()?;
    let mut bad_glyphs: HashMap<String, HashSet<(GlyphId, String)>> = HashMap::new();
    for glyph in f.all_glyphs() {
        if let Ok(Some(variation)) = gvar.glyph_variation_data(glyph) {
            let tuples = variation.tuples();
            for tuple in tuples {
                let duplex_axis_is_affected = axis_indices
                    .iter()
                    .map(|idx| {
                        (
                            idx,
                            tuple.peak().values().get(*idx).map(|v| v.get().to_f32()),
                        )
                    })
                    .filter(|(_axis, value)| value.is_some() && *value != Some(0.0))
                    .map(|(axis, _value)| axis)
                    .collect::<Vec<_>>();
                if !duplex_axis_is_affected.is_empty() {
                    // Find the index of the phantom points in this glyph.
                    let glyf_glyph = f.get_glyf_glyph(glyph)?.ok_or_else(|| {
                        CheckError::Error(format!("Glyph {} not found in glyf table", glyph))
                    })?;
                    let point_count = match glyf_glyph {
                        Glyph::Simple(g) => g.points().count(),
                        Glyph::Composite(g) => g.components().count(),
                    };
                    let hadvance_point = point_count + 1;

                    let position = tuple
                        .peak()
                        .values()
                        .iter()
                        .zip(f.font().axes().iter().map(|ax| ax.tag()))
                        .map(|(value, axis)| format!("{}={}", axis, value.get().to_f32()))
                        .join(", ");
                    let points = tuple.deltas().collect::<Vec<_>>();
                    // println!("Points: {:?}", points);
                    // println!("Looking for position: {}", hadvance_point);
                    let advance_delta = points
                        .iter()
                        .find(|p| p.position == hadvance_point as u16)
                        .map(|p| p.x_delta);
                    if advance_delta.is_some() && advance_delta != Some(0) {
                        for axis in duplex_axis_is_affected {
                            #[allow(clippy::unwrap_used)] // We know the axis exists
                            let axis_tag = f.font().axes().get(*axis).unwrap().tag().to_string();
                            bad_glyphs
                                .entry(axis_tag)
                                .or_default()
                                .insert((glyph, position.to_string()));
                        }
                    }
                }
            }
        }
    }

    for (tag, glyphs) in bad_glyphs.iter() {
        let glyphs_list = glyphs
            .iter()
            .map(|(g, pos)| f.glyph_name_for_id_synthesise(*g) + " at position " + pos);
        problems.push(Status::fail(
            &format!("{}-causes-reflow", tag.to_lowercase()),
            &format!(
                "The following glyphs have variation in horizontal advance due to duplexed axis {}:\n{}",
                tag, bullet_list(context, glyphs_list)
            ),
        ));
    }

    // Determine if any kerning rules vary the horizontal advance.
    // This is going to get grubby.
    // I mean, even grubbier than the Python implementation was.
    if let Some(Ok(varstore)) = f.font().gdef().ok().and_then(|gdef| gdef.item_var_store()) {
        let regions = varstore.variation_region_list()?.variation_regions();
        let mut effective_regions = HashSet::new();
        for index in axis_indices.iter() {
            for (ix, region) in regions.iter().enumerate() {
                let region = region?;
                let axis_tent = region
                    .region_axes()
                    .get(*index)
                    .ok_or_else(|| CheckError::Error("Something went wrong".to_string()))?;
                let effective = axis_tent.start_coord() != axis_tent.peak_coord()
                    || axis_tent.peak_coord() != axis_tent.end_coord();
                if effective {
                    effective_regions.insert(ix as u16);
                }
            }

            if !effective_regions.is_empty() {
                // println!("Effective regions: {:?}", effective_regions);
                let kerns_with_region = f.process_kerning(
                    &|pp1| pairs_with_region_1(pp1, &effective_regions, &varstore),
                    &|pp2| pairs_with_region_2(pp2, &effective_regions, &varstore),
                )?;
                if let Some((left, right)) = kerns_with_region.first() {
                    problems.push(Status::fail("duplexed-kern-causes-reflow", 
                        &format!(
                            "Kerning rules cause variation in horizontal advance on a duplexed axis (e.g. {}/{})",
                            f.glyph_name_for_id_synthesise(*left),
                            f.glyph_name_for_id_synthesise(*right)
                        )
                    ));
                }
            }
        }
    }
    return_result(problems)
}

fn pairs_with_region_1(
    pp1: PairPosFormat1,
    effective_regions: &HashSet<u16>,
    var_store: &ItemVariationStore,
) -> Result<Vec<(GlyphId, GlyphId)>, ReadError> {
    let mut results = vec![];
    let coverage = pp1.coverage()?;
    for (left, pairset) in coverage.iter().zip(pp1.pair_sets().iter()) {
        let pairset = pairset?;
        for pairrecord in pairset.pair_value_records().iter() {
            let pairrecord = pairrecord?;
            let value_record = pairrecord.value_record1();
            if grovel_item_variation_store(
                value_record,
                pairset.offset_data(),
                var_store,
                effective_regions,
            )? {
                results.push((
                    GlyphId::from(left),
                    GlyphId::from(pairrecord.second_glyph()),
                ));
            }
        }
    }
    Ok(results)
}

fn pairs_with_region_2(
    pp2: PairPosFormat2,
    effective_regions: &HashSet<u16>,
    var_store: &ItemVariationStore,
) -> Result<Vec<(GlyphId, GlyphId)>, ReadError> {
    let mut results = vec![];
    let class1 = pp2.class_def1()?;
    let class2 = pp2.class_def2()?;
    let mut glyphs_per_class1: HashMap<_, HashSet<GlyphId>> = HashMap::new();
    for (gid, class) in class1.iter() {
        if class == 0 {
            continue;
        } // Gibberish
        let glyphs = glyphs_per_class1.entry(class).or_insert_with(HashSet::new);
        glyphs.insert(GlyphId::from(gid));
    }

    let mut glyphs_per_class2: HashMap<_, HashSet<GlyphId>> = HashMap::new();
    for (gid, class) in class2.iter() {
        if class == 0 {
            continue;
        }
        let glyphs = glyphs_per_class2.entry(class).or_insert_with(HashSet::new);
        glyphs.insert(GlyphId::from(gid));
    }

    for (class1_id, record) in pp2.class1_records().iter().enumerate() {
        let record = record?;
        let glyphs1 = glyphs_per_class1.get(&(class1_id as u16));

        for (class2_id, class2_rec) in record.class2_records().iter().enumerate() {
            let class2_rec = class2_rec?;
            let value_record = class2_rec.value_record1();
            let glyphs2 = glyphs_per_class2.get(&(class2_id as u16));
            if grovel_item_variation_store(
                value_record,
                pp2.offset_data(),
                var_store,
                effective_regions,
            )? {
                for left in glyphs1.map(|x| x.iter()).into_iter().flatten() {
                    for right in glyphs2.map(|x| x.iter()).into_iter().flatten() {
                        results.push((*left, *right));
                    }
                }
            }
        }
    }
    Ok(results)
}

fn grovel_item_variation_store(
    value_record: &read_fonts::tables::gpos::ValueRecord,
    offset_data: FontData<'_>,
    var_store: &ItemVariationStore,
    effective_regions: &HashSet<u16>,
) -> Result<bool, ReadError> {
    if let Some(device_index) = value_record.x_advance_device(offset_data) {
        if let Some(variation) = device_index.map(|v| match v {
            Device(_) => None,
            VariationIndex(table_ref) => Some(table_ref),
        })? {
            if let Some(variation_data) = var_store
                .item_variation_data()
                .get(variation.delta_set_outer_index().into())
                .transpose()?
            {
                let regions = variation_data.region_indexes();
                if regions
                    .iter()
                    .any(|region| effective_regions.contains(&region.get()))
                {
                    let deltas = variation_data.delta_set(variation.delta_set_inner_index());
                    let effective_deltas = deltas
                        .zip(regions.iter())
                        .filter(|(_, region)| effective_regions.contains(&region.get()))
                        .map(|(delta, _)| delta)
                        .collect::<Vec<_>>();
                    if effective_deltas.iter().any(|x| x != &0) {
                        return Ok(true);
                    }
                }
            }
        }
    }
    Ok(false)
}
