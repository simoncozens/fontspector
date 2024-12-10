use std::collections::HashSet;

use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert, GetSubstitutionMap};
use read_fonts::{tables::gsub::SubstitutionLookupList, ReadError, TableProvider};
use skrifa::{GlyphId16, MetadataProvider};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

#[check(
    id = "missing_small_caps_glyphs",
    rationale = "
        Ensure small caps glyphs are available if a font declares smcp or c2sc OT features.",
    proposal = "https://github.com/fonttools/fontbakery/issues/3154",
    title = "Ensure small caps glyphs are available"
)]
fn missing_small_caps_glyphs(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    // Skip if no smcp or c2sc
    let smcp_lookups = f
        .feature_records(true)
        .filter(|(r, _l)| r.feature_tag() == "smcp")
        .flat_map(|(_r, l)| l)
        .flat_map(|l| l.lookup_list_indices())
        .collect::<Vec<_>>();

    let c2sc_lookups = f
        .feature_records(true)
        .filter(|(r, _l)| r.feature_tag() == "c2sc")
        .flat_map(|(_r, l)| l)
        .flat_map(|l| l.lookup_list_indices())
        .collect::<Vec<_>>();
    skip!(
        smcp_lookups.is_empty() && c2sc_lookups.is_empty(),
        "no-smcp-c2sc",
        "No smcp or c2sc features"
    );
    let mut problems = vec![];

    for (lookups, category, feature, error_code) in &[
        (
            smcp_lookups,
            GeneralCategory::LowercaseLetter,
            "smcp",
            "missing-smcp-lowercase",
        ),
        (
            c2sc_lookups,
            GeneralCategory::UppercaseLetter,
            "c2sc",
            "missing-c2sc-uppercase",
        ),
    ] {
        let mut glyphset: HashSet<GlyphId16> = f
            .font()
            .charmap()
            .mappings()
            .filter_map(|(c, g)| {
                char::from_u32(c).and_then(|c| {
                    if c.general_category() == *category {
                        GlyphId16::try_from(g).ok()
                    } else {
                        None
                    }
                })
            })
            .collect();

        remove_lhs_glyphs(
            &mut glyphset,
            &f.font().gsub()?.lookup_list()?,
            lookups.iter().map(|i| i.get()),
        )?;

        if !glyphset.is_empty() {
            problems.push(Status::fail(
                error_code,
                &format!(
                    "The following letters did not take part in {} substitutions:\n{}",
                    feature,
                    bullet_list(
                        context,
                        glyphset.iter().map(|g| f.glyph_name_for_id_synthesise(*g))
                    )
                ),
            ));
        }
    }

    return_result(problems)
}

fn remove_lhs_glyphs(
    glyphset: &mut HashSet<GlyphId16>,
    lookups: &SubstitutionLookupList,
    lookups_indices: impl Iterator<Item = u16>,
) -> Result<(), ReadError> {
    for i in lookups_indices {
        let lookup = lookups.lookups().get(i as usize)?;
        for (lhs, _rhs) in lookup.subtables()?.substitutions()?.iter() {
            for gid in lhs {
                glyphset.remove(gid);
            }
        }
    }
    Ok(())
}
