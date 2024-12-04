use std::collections::{HashMap, HashSet};

use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert, GetSubstitutionMap};
use itertools::Itertools;
use read_fonts::{
    tables::gpos::{PairPosFormat1, PairPosFormat2},
    ReadError, TableProvider,
};
use skrifa::{GlyphId, MetadataProvider};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

const _EXCLUDED: [u32; 12] = [
    0x0600, // Arabic
    0x0601, // Arabic
    0x0602, // Arabic
    0x0603, // Arabic
    0x0604, // Arabic
    0x06DD, // Arabic
    0x0890, // Arabic
    0x0891, // Arabic
    0x0605, // Arabic
    0x08E2, // Arabic
    0x2044, // General Punctuation
    0x2215, // Mathematical Operators
];

fn is_symbol(cp: &u32) -> bool {
    char::from_u32(*cp).map_or(false, |c| {
        matches!(
            c.general_category(),
            GeneralCategory::DecimalNumber
                | GeneralCategory::OtherNumber
                | GeneralCategory::LetterNumber
                | GeneralCategory::MathSymbol
                | GeneralCategory::CurrencySymbol
        )
    })
}
const NUMERALS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

#[check(
    id = "tabular_kerning",
    rationale = "
        
        Tabular glyphs should not have kerning, as they are meant to be used in tables.

        This check looks for kerning in:
        - all glyphs in a font in combination with tabular numerals;
        - tabular symbols in combination with tabular numerals.

        \"Tabular symbols\" is defined as:
        - for fonts with a \"tnum\" feature, all \"tnum\" substitution target glyphs;
        - for fonts without a \"tnum\" feature, all glyphs that have the same width
        as the tabular numerals, but limited to numbers, math and currency symbols.

        This check may produce false positives for fonts with no \"tnum\" feature
        and with equal-width numerals (and other same-width symbols) that are
        not intended to be used as tabular numerals.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4440",
    title = "Check tabular widths don't have kerning."
)]
fn tabular_kerning(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    let tnum_lookups = f
        .feature_records(true)
        .filter(|(r, _l)| r.feature_tag() == "tnum")
        .flat_map(|(_r, l)| l)
        .flat_map(|l| l.lookup_list_indices())
        .collect::<Vec<_>>();
    skip!(
        !f.has_feature(false, "kern"),
        "no-kern",
        "Font has no kern feature"
    );
    let numeral_glyphs = NUMERALS
        .iter()
        .filter_map(|c| f.font().charmap().map(*c))
        .collect::<HashSet<_>>();
    skip!(
        numeral_glyphs.len() < 10,
        "no-numerals",
        "Font has no numerals at all"
    );
    let unicode_per_glyph = f
        .font()
        .charmap()
        .mappings()
        .map(|(c, g)| (g, c))
        .collect::<HashMap<_, _>>();

    let (tabular_numerals, tabular_glyphs) = if f.has_feature(true, "tnum") {
        // tabular glyphs is anything on the RHS of a tnum
        let mut tabular_glyphs = HashSet::new();
        let mut tabular_numerals = numeral_glyphs.clone();
        let lookups = f.font().gsub()?.lookup_list()?;
        for i in tnum_lookups {
            let lookup = lookups.lookups().get(i.get() as usize)?;
            let substitutions = lookup.subtables()?.substitutions()?;
            for (lhs, rhs) in substitutions.iter() {
                for gid in rhs {
                    if lhs.len() == 1 && numeral_glyphs.contains(&lhs[0].into()) {
                        tabular_numerals.remove(&(lhs[0].into()));
                        tabular_numerals.insert(GlyphId::from(*gid));
                    } else {
                        tabular_glyphs.insert(GlyphId::from(*gid));
                    }
                }
            }
        }
        (tabular_numerals, tabular_glyphs)
    } else {
        let hmtx = f.font().hmtx()?;
        let widths = numeral_glyphs
            .iter()
            .flat_map(|gid| hmtx.advance(*gid))
            .collect::<HashSet<_>>();
        if widths.len() == 1 {
            #[allow(clippy::unwrap_used)] // We know there's one element
            let tabular_width = *widths.iter().next().unwrap();
            let others_with_same_width = hmtx
                .h_metrics()
                .iter()
                .enumerate()
                .map(|(gid, metric)| (GlyphId::from(gid as u32), metric))
                .filter(|(gid, metric)| {
                    metric.advance() == tabular_width && !numeral_glyphs.contains(gid)
                })
                .map(|(gid, _)| gid);
            let tabular_glyphs: HashSet<_> = others_with_same_width
                .filter(|gid| unicode_per_glyph.get(gid).map_or(false, is_symbol))
                .collect();
            (numeral_glyphs, tabular_glyphs)
        } else {
            (HashSet::new(), HashSet::new())
        }
    };
    skip!(
        tabular_numerals.is_empty(),
        "no-tabular-numerals",
        "Font has no tabular numerals"
    );

    // We're not going to use a shaper to do the kerning since implementing
    // the nominal_glyph_func hack in Rust is a bit of a pain; instead we'll
    // just check for any GPOS PairPositioning rules between the involved glyphs.
    // Faster too.
    let kerning: Vec<(HashSet<GlyphId>, HashSet<GlyphId>)> =
        f.process_kerning(involved_pairs_format1, involved_pairs_format2)?;

    let has_kerning = |a: &GlyphId, b: &GlyphId| {
        kerning
            .iter()
            .any(|(lhs, rhs)| lhs.contains(a) && rhs.contains(b))
    };

    // println!(
    //     "Tabular numerals: {:?}",
    //     tabular_numerals
    //         .iter()
    //         .map(|x| f.glyph_name_for_id_synthesise(*x))
    //         .collect::<Vec<_>>()
    // );
    // println!(
    //     "Tabular symbols: {:?}",
    //     tabular_glyphs
    //         .iter()
    //         .map(|x| f.glyph_name_for_id_synthesise(*x))
    //         .collect::<Vec<_>>()
    // );

    for (a, b) in tabular_glyphs
        .iter()
        .chain(tabular_numerals.iter())
        .cartesian_product(tabular_numerals.iter())
    {
        if has_kerning(a, b) {
            problems.push(Status::fail(
                "has-tabular-kerning",
                &format!(
                    "Kerning between {} and {}",
                    f.glyph_name_for_id_synthesise(*a),
                    f.glyph_name_for_id_synthesise(*b),
                ),
            ));
        }
    }
    return_result(problems)
}

type PairSlot = (HashSet<GlyphId>, HashSet<GlyphId>);
fn involved_pairs_format1(pp1: PairPosFormat1) -> Result<Vec<PairSlot>, ReadError> {
    let mut results = vec![];
    let coverage = pp1.coverage()?;
    for (left, pairset) in coverage.iter().zip(pp1.pair_sets().iter()) {
        let pairset = pairset?;
        for pairrecord in pairset.pair_value_records().iter() {
            let pairrecord = pairrecord?;
            if let Some(x) = pairrecord.value_record1().x_advance() {
                if x != 0 {
                    results.push((
                        vec![GlyphId::from(left)].into_iter().collect(),
                        vec![GlyphId::from(pairrecord.second_glyph())]
                            .into_iter()
                            .collect(),
                    ));
                }
            }
        }
    }
    Ok(results)
}
fn involved_pairs_format2(pp2: PairPosFormat2) -> Result<Vec<PairSlot>, ReadError> {
    let class1 = pp2.class_def1()?;
    let class2 = pp2.class_def2()?;
    let mut results = vec![];
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

    // I'm rethinking my life choices here. Maybe shaping would have been easier.
    for (class1_id, record) in pp2.class1_records().iter().enumerate() {
        let record = record?;
        let glyphs1 = glyphs_per_class1.get(&(class1_id as u16));

        for (class2_id, class2_rec) in record.class2_records().iter().enumerate() {
            if let Some(x) = class2_rec?.value_record1().x_advance() {
                if x != 0 {
                    let glyphs2 = glyphs_per_class2.get(&(class2_id as u16));
                    results.push((
                        glyphs1.cloned().unwrap_or(HashSet::new()),
                        glyphs2.cloned().unwrap_or(HashSet::new()),
                    ));
                }
            }
        }
    }
    Ok(results)
}
