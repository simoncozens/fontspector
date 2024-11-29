use std::collections::HashSet;

use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use itertools::Itertools;
use read_fonts::{
    tables::{
        colr::Paint,
        glyf::Glyph::{Composite, Simple},
        gsub::{
            AlternateSubstFormat1, ExtensionSubstFormat1, LigatureSubstFormat1,
            MultipleSubstFormat1, ReverseChainSingleSubstFormat1, SingleSubst,
            SubstitutionSubtables,
        },
        layout::Subtables,
    },
    ReadError, TableProvider,
};
use skrifa::{charmap::MapVariant, GlyphId, MetadataProvider};

type SubSubtables<'a, T> = Subtables<'a, T, ExtensionSubstFormat1<'a, T>>;

fn handle_gsub1<'a>(
    st: &SubSubtables<'a, SingleSubst<'a>>,
    glyphs: &mut HashSet<GlyphId>,
) -> Result<(), ReadError> {
    for subtable in st.iter() {
        match subtable? {
            SingleSubst::Format1(table_ref) => {
                let delta = table_ref.delta_glyph_id() as i32;
                let produced = table_ref
                    .coverage()?
                    .iter()
                    .map(|old_gid| GlyphId::from((old_gid.to_u32() as i32 + delta) as u16));
                for gid in produced {
                    glyphs.remove(&gid);
                }
            }
            SingleSubst::Format2(table_ref) => {
                for produced in table_ref.substitute_glyph_ids() {
                    glyphs.remove(&produced.get().into());
                }
            }
        }
    }
    Ok(())
}

fn handle_gsub2<'a>(
    st: &SubSubtables<'a, MultipleSubstFormat1<'a>>,
    glyphs: &mut HashSet<GlyphId>,
) -> Result<(), ReadError> {
    for subtable in st.iter() {
        for sequence in subtable?.sequences().iter().flatten() {
            for glyph in sequence.substitute_glyph_ids() {
                glyphs.remove(&glyph.get().into());
            }
        }
    }
    Ok(())
}

fn handle_gsub3<'a>(
    st: &SubSubtables<'a, AlternateSubstFormat1<'a>>,
    glyphs: &mut HashSet<GlyphId>,
) -> Result<(), ReadError> {
    for subtable in st.iter() {
        for sequence in subtable?.alternate_sets().iter().flatten() {
            for glyph in sequence.alternate_glyph_ids() {
                glyphs.remove(&glyph.get().into());
            }
        }
    }
    Ok(())
}
fn handle_gsub4<'a>(
    st: &SubSubtables<'a, LigatureSubstFormat1<'a>>,
    glyphs: &mut HashSet<GlyphId>,
) -> Result<(), ReadError> {
    for subtable in st.iter() {
        for sequence in subtable?.ligature_sets().iter().flatten() {
            for glyph in sequence.ligatures().iter().flatten() {
                glyphs.remove(&glyph.ligature_glyph().into());
            }
        }
    }
    Ok(())
}

fn handle_gsub7<'a>(
    st: &SubSubtables<'a, ReverseChainSingleSubstFormat1<'a>>,
    glyphs: &mut HashSet<GlyphId>,
) -> Result<(), ReadError> {
    for subtable in st.iter() {
        for gid in subtable?.substitute_glyph_ids() {
            glyphs.remove(&gid.get().into());
        }
    }
    Ok(())
}
#[check(
    id = "unreachable_glyphs",
    rationale = "
        Glyphs are either accessible directly through Unicode codepoints or through
        substitution rules.

        In Color Fonts, glyphs are also referenced by the COLR table. And mathematical
        fonts also reference glyphs via the MATH table.

        Any glyphs not accessible by these means are redundant and serve only
        to increase the font's file size.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3160",
    title = "Check font contains no unreachable glyphs"
)]
fn unreachable_glyphs(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut glyphs = f.all_glyphs().collect::<HashSet<_>>();
    // cmap
    for (_, gid) in f.font().charmap().mappings() {
        glyphs.remove(&gid);
    }
    // UVS
    for (_, _, map) in f.font().charmap().variant_mappings() {
        match map {
            MapVariant::UseDefault => {}
            MapVariant::Variant(glyph_id) => {
                glyphs.remove(&glyph_id);
            }
        }
    }

    // No math table support yet, working on it...
    // if let Some(Ok(math)) = f.font().math() {}

    if let Ok(colr) = f.font().colr() {
        // COLRv0
        if let Some(Ok(recs)) = colr.layer_records() {
            for rec in recs {
                glyphs.remove(&rec.glyph_id().into());
            }
        }
        // COLRv1
        if let Some(Ok(base_glyph_array)) = colr.base_glyph_records() {
            for rec in base_glyph_array {
                glyphs.remove(&rec.glyph_id().into());
            }
        }
        if let Some(Ok(base_glyph_list)) = colr.base_glyph_list() {
            for rec in base_glyph_list.base_glyph_paint_records() {
                let paint = rec.paint(base_glyph_list.offset_data())?;
                match paint {
                    Paint::Glyph(paint_glyph) => {
                        glyphs.remove(&paint_glyph.glyph_id().into());
                    }
                    Paint::ColrGlyph(table_ref) => {
                        glyphs.remove(&table_ref.glyph_id().into());
                    }
                    _ => {}
                }
                glyphs.remove(&rec.glyph_id().into());
            }
        }
        if let Some(Ok(layer_list)) = colr.layer_list() {
            for rec in layer_list.paints().iter().flatten() {
                match rec {
                    Paint::Glyph(paint_glyph) => {
                        glyphs.remove(&paint_glyph.glyph_id().into());
                    }
                    Paint::ColrGlyph(table_ref) => {
                        glyphs.remove(&table_ref.glyph_id().into());
                    }
                    _ => {}
                }
            }
        }
    }

    // GSUB productions
    if let Ok(gsub) = f.font().gsub() {
        for lookup in gsub.lookup_list()?.lookups().iter().flatten() {
            match lookup.subtables()? {
                SubstitutionSubtables::Single(st) => handle_gsub1(&st, &mut glyphs),
                SubstitutionSubtables::Multiple(st) => handle_gsub2(&st, &mut glyphs),
                SubstitutionSubtables::Alternate(st) => handle_gsub3(&st, &mut glyphs),
                SubstitutionSubtables::Ligature(st) => handle_gsub4(&st, &mut glyphs),
                // The contextual ones are not needed because all they do is call other
                // lookups in the list.
                SubstitutionSubtables::Contextual(_) => Ok(()),
                SubstitutionSubtables::ChainContextual(_) => Ok(()),
                SubstitutionSubtables::Reverse(st) => handle_gsub7(&st, &mut glyphs),
            }?;
        }
    }
    // Remove components used in TrueType table
    for glyph in f
        .all_glyphs()
        .flat_map(|gid| f.get_glyf_glyph(gid))
        .flatten()
    {
        match glyph {
            Simple(_) => {}
            Composite(composite) => {
                for component in composite.components() {
                    glyphs.remove(&component.glyph.into());
                }
            }
        }
    }

    glyphs.remove(&GlyphId::from(0u32));

    if glyphs.is_empty() {
        Ok(Status::just_one_pass())
    } else {
        Ok(Status::just_one_warn(
            "unreachable-glyphs",
            &format!(
                "The following glyphs could not be reached by codepoint or substitution rules:\n\n{}",
                bullet_list(
                    context,
                    glyphs
                        .iter()
                        .sorted()
                        .map(|gid| f.glyph_name_for_id_synthesise(*gid))
                )
            ),
        ))
    }
}
