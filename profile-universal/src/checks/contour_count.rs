use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use fontspector_checkapi::{
    pens::ContourCountPen, prelude::*, testfont, FileTypeConvert, DEFAULT_LOCATION,
};
use skrifa::MetadataProvider;

const DATA_JSON: &str = include_str!("../../data/desired_glyph_data.json");

#[allow(clippy::unwrap_used)]
static GLYPHS_BY_NAME: LazyLock<HashMap<String, HashSet<usize>>> = LazyLock::new(|| {
    let data: serde_json::Map<String, serde_json::Value> = serde_json::from_str(DATA_JSON).unwrap();
    let value = data.get("by_name").unwrap().as_object().unwrap();
    let mut map = HashMap::new();
    for (name, indices) in value {
        let indices = indices.as_array().unwrap();
        let indices = indices
            .iter()
            .map(|v| v.as_u64().unwrap() as usize)
            .collect();
        map.insert(name.clone(), indices);
    }
    map
});

#[allow(clippy::unwrap_used)]
static GLYPHS_BY_UNICODE: LazyLock<HashMap<u32, HashSet<usize>>> = LazyLock::new(|| {
    let data: serde_json::Map<String, serde_json::Value> = serde_json::from_str(DATA_JSON).unwrap();
    let value = data.get("by_unicode").unwrap().as_object().unwrap();
    let mut map = HashMap::new();
    for (codepoint, indices) in value {
        let indices = indices.as_array().unwrap();
        let indices = indices
            .iter()
            .map(|v| v.as_u64().unwrap() as usize)
            .collect();
        map.insert(codepoint.parse::<u32>().unwrap(), indices);
    }
    map
});

#[check(
    id = "contour_count",
    rationale = "
        
        Visually QAing thousands of glyphs by hand is tiring. Most glyphs can only
        be constructured in a handful of ways. This means a glyph's contour count
        will only differ slightly amongst different fonts, e.g a 'g' could either
        be 2 or 3 contours, depending on whether its double story or single story.

        However, a quotedbl should have 2 contours, unless the font belongs
        to a display family.

        This check currently does not cover variable fonts because there's plenty
        of alternative ways of constructing glyphs with multiple outlines for each
        feature in a VarFont. The expected contour count data for this check is
        currently optimized for the typical construction of glyphs in static fonts.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Check if each glyph has the recommended amount of contours."
)]
fn contour_count(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    let mut bad_glyphs = vec![];
    let mut zero_contours = vec![];
    let reverse_map = f
        .font()
        .charmap()
        .mappings()
        .map(|(k, v)| (v, k))
        .collect::<HashMap<_, _>>();
    for glyph in f.all_glyphs() {
        if let Some(codepoint) = reverse_map.get(&glyph) {
            if let Some(data) = GLYPHS_BY_UNICODE.get(codepoint) {
                let name = f.glyph_name_for_id_synthesise(glyph);
                let mut pen = ContourCountPen::new();
                f.draw_glyph(glyph, &mut pen, DEFAULT_LOCATION)?;
                let count = pen.contour_count();
                if count == 0 && !data.contains(&count) {
                    zero_contours.push(name);
                } else if !data.contains(&count) {
                    bad_glyphs.push(format!(
                        "{} (U+{:04X}): found {}, expected one of: {:?}",
                        name, codepoint, count, data
                    ));
                }
            }
        } else {
            let name = f.glyph_name_for_id_synthesise(glyph);
            if let Some(data) = GLYPHS_BY_NAME.get(&name) {
                let mut pen = ContourCountPen::new();
                f.draw_glyph(glyph, &mut pen, DEFAULT_LOCATION)?;
                let count = pen.contour_count();
                if count == 0 && !data.contains(&count) {
                    zero_contours.push(name);
                } else if !data.contains(&count) {
                    bad_glyphs.push(format!(
                        "{} (unencoded): found {}, expected one of: {:?}",
                        name, count, data
                    ));
                }
            }
        }
    }
    if !bad_glyphs.is_empty() {
        problems.push(Status::warn(
            "contour-count",
            &format!(
                "This check inspects the glyph outlines and detects the total number of contours in each of them. The expected values are
     infered from the typical ammounts of contours observed in a
     large collection of reference font families. The divergences
     listed below may simply indicate a significantly different
     design on some of your glyphs. On the other hand, some of these
     may flag actual bugs in the font such as glyphs mapped to an
     incorrect codepoint. Please consider reviewing the design and
     codepoint assignment of these to make sure they are correct.\n\n
    The following glyphs do not have the recommended number of contours:\n{}",
                bullet_list(context, &bad_glyphs),
            ),
        ));
    }
    if !zero_contours.is_empty() {
        problems.push(Status::fail(
            "no-contour",
            &format!(
                "The following glyphs have no contours even though they were expected to have some:\n{}",
                bullet_list(context, &zero_contours),
            ),
        ));
    }
    return_result(problems)
}
