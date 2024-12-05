// Code to make GSUB tables easier to work with
use read_fonts::{
    tables::{
        gsub::{
            AlternateSubstFormat1, ExtensionSubstFormat1, LigatureSubstFormat1,
            MultipleSubstFormat1, ReverseChainSingleSubstFormat1, SingleSubst,
            SubstitutionSubtables,
        },
        layout::Subtables,
    },
    ReadError,
};
use skrifa::GlyphId16;

/// A map of substitutions, input glyphs on the left, output glyphs on the right
pub type SubstitutionMap = Vec<(Vec<GlyphId16>, Vec<GlyphId16>)>;

/// A trait to get a substitution map from a GSUB subtable
pub trait GetSubstitutionMap {
    /// Get the substitution map from the subtable
    fn substitutions(&self) -> Result<SubstitutionMap, ReadError>;
}

/// This type isn't public in `read-fonts` so copy it in
type SubSubtables<'a, T> = Subtables<'a, T, ExtensionSubstFormat1<'a, T>>;

impl<'a> GetSubstitutionMap for SubSubtables<'a, SingleSubst<'a>> {
    fn substitutions(&self) -> Result<Vec<(Vec<GlyphId16>, Vec<GlyphId16>)>, ReadError> {
        let mut result = vec![];
        for subtable in self.iter() {
            match subtable? {
                SingleSubst::Format1(table_ref) => {
                    let delta = table_ref.delta_glyph_id() as i32;
                    result.extend(table_ref.coverage()?.iter().map(|old_gid| {
                        let new_gid = GlyphId16::from((old_gid.to_u32() as i32 + delta) as u16);
                        (vec![old_gid], vec![new_gid])
                    }));
                }
                SingleSubst::Format2(table_ref) => {
                    result.extend(
                        table_ref
                            .coverage()?
                            .iter()
                            .zip(table_ref.substitute_glyph_ids().iter().map(|gid| gid.get()))
                            .map(|(old_gid, new_gid)| (vec![old_gid], vec![new_gid])),
                    );
                }
            }
        }
        Ok(result)
    }
}

impl<'a> GetSubstitutionMap for SubSubtables<'a, MultipleSubstFormat1<'a>> {
    fn substitutions(&self) -> Result<Vec<(Vec<GlyphId16>, Vec<GlyphId16>)>, ReadError> {
        let mut result = vec![];
        for subtable in self.iter() {
            let subtable = subtable?;
            for (old_gid, sequence) in subtable
                .coverage()?
                .iter()
                .zip(subtable.sequences().iter().flatten())
            {
                let new_gids = sequence
                    .substitute_glyph_ids()
                    .iter()
                    .map(|gid| gid.get())
                    .collect();
                result.push((vec![old_gid], new_gids));
            }
        }
        Ok(result)
    }
}

impl<'a> GetSubstitutionMap for SubSubtables<'a, AlternateSubstFormat1<'a>> {
    fn substitutions(&self) -> Result<Vec<(Vec<GlyphId16>, Vec<GlyphId16>)>, ReadError> {
        let mut result = vec![];
        for subtable in self.iter() {
            let subtable = subtable?;
            for (old_gid, sequence) in subtable
                .coverage()?
                .iter()
                .zip(subtable.alternate_sets().iter().flatten())
            {
                let new_gids = sequence
                    .alternate_glyph_ids()
                    .iter()
                    .map(|gid| gid.get())
                    .collect();
                result.push((vec![old_gid], new_gids));
            }
        }
        Ok(result)
    }
}

impl<'a> GetSubstitutionMap for SubSubtables<'a, LigatureSubstFormat1<'a>> {
    fn substitutions(&self) -> Result<Vec<(Vec<GlyphId16>, Vec<GlyphId16>)>, ReadError> {
        let mut result = vec![];
        for subtable in self.iter() {
            let subtable = subtable?;
            for (first_gid, sequence) in subtable
                .coverage()?
                .iter()
                .zip(subtable.ligature_sets().iter().flatten())
            {
                for ligature in sequence.ligatures().iter().flatten() {
                    let remaining_gids = ligature.component_glyph_ids().iter().map(|gid| gid.get());
                    let new_gid = ligature.ligature_glyph();
                    let mut lhs = vec![first_gid];
                    lhs.extend(remaining_gids);
                    result.push((lhs, vec![new_gid]));
                }
            }
        }
        Ok(result)
    }
}

impl<'a> GetSubstitutionMap for SubSubtables<'a, ReverseChainSingleSubstFormat1<'a>> {
    fn substitutions(&self) -> Result<Vec<(Vec<GlyphId16>, Vec<GlyphId16>)>, ReadError> {
        let mut result = vec![];
        for subtable in self.iter() {
            let subtable = subtable?;
            for (lgid, rgid) in subtable
                .coverage()?
                .iter()
                .zip(subtable.substitute_glyph_ids().iter())
            {
                result.push((vec![lgid], vec![rgid.get()]));
            }
        }
        Ok(result)
    }
}
impl GetSubstitutionMap for SubstitutionSubtables<'_> {
    fn substitutions(&self) -> Result<Vec<(Vec<GlyphId16>, Vec<GlyphId16>)>, ReadError> {
        match self {
            SubstitutionSubtables::Single(st) => st.substitutions(),
            SubstitutionSubtables::Multiple(st) => st.substitutions(),
            SubstitutionSubtables::Alternate(st) => st.substitutions(),
            SubstitutionSubtables::Ligature(st) => st.substitutions(),
            SubstitutionSubtables::Reverse(st) => st.substitutions(),
            _ => Ok(vec![]),
        }
    }
}
