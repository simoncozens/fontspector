use std::collections::{HashMap, HashSet};

use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert, TestFont};
use read_fonts::{tables::gdef::GlyphClassDef, ReadError, TableProvider};
use skrifa::{GlyphId, MetadataProvider};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

#[check(
    id = "opentype/monospace",
    rationale = "
        There are various metadata in the OpenType spec to specify if a font is
        monospaced or not. If the font is not truly monospaced, then no monospaced
        metadata should be set (as sometimes they mistakenly are...)

        Requirements for monospace fonts:

        * post.isFixedPitch - \"Set to 0 if the font is proportionally spaced,
          non-zero if the font is not proportionally spaced (monospaced)\"
          (https://www.microsoft.com/typography/otspec/post.htm)

        * hhea.advanceWidthMax must be correct, meaning no glyph's width value
          is greater. (https://www.microsoft.com/typography/otspec/hhea.htm)

        * OS/2.panose.bProportion must be set to 9 (monospace) on latin text fonts.

        * OS/2.panose.bSpacing must be set to 3 (monospace) on latin hand written
          or latin symbol fonts.

        * Spec says: \"The PANOSE definition contains ten digits each of which currently
          describes up to sixteen variations. Windows uses bFamilyType, bSerifStyle
          and bProportion in the font mapper to determine family type. It also uses
          bProportion to determine if the font is monospaced.\"
          (https://www.microsoft.com/typography/otspec/os2.htm#pan
           https://monotypecom-test.monotype.de/services/pan2)

        * OS/2.xAvgCharWidth must be set accurately.
          \"OS/2.xAvgCharWidth is used when rendering monospaced fonts,
          at least by Windows GDI\"
          (http://typedrawers.com/discussion/comment/15397/#Comment_15397)

        Also we should report an error for glyphs not of average width.


        Please also note:

        Thomas Phinney told us that a few years ago (as of December 2019), if you gave
        a font a monospace flag in Panose, Microsoft Word would ignore the actual
        advance widths and treat it as monospaced.

        Source: https://typedrawers.com/discussion/comment/45140/#Comment_45140
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Checking correctness of monospaced metadata."
)]
fn monospace(t: &Testable, context: &Context) -> CheckFnResult {
    let font = testfont!(t);
    for required in [b"glyf", b"hhea", b"hmtx", b"OS/2", b"post"] {
        if !font.has_table(required) {
            return Ok(Status::just_one_fail(
                "missing-table",
                &format!("Font is missing a required table: {:?}", required),
            ));
        }
    }

    let statistics = glyph_metrics_stats(&font)?;
    let mut problems = vec![];
    // Funny place to be checking it but OK
    let advance_width_max = font.font().hhea()?.advance_width_max().to_u16();
    if advance_width_max != statistics.width_max {
        problems.push(Status::fail(
            "bad-advanceWidthMax",
            &format!(
                "Value of hhea.advanceWidthMax should be set to {} but got {} instead.",
                statistics.width_max, advance_width_max,
            ),
        ));
    }
    let post_isfixedpitch = font.font().post()?.is_fixed_pitch();
    let panose = font.font().os2()?.panose_10();

    if statistics.seems_monospaced {
        let number_of_h_metrics = font.font().hhea()?.number_of_long_metrics();
        if number_of_h_metrics != 3 {
            problems.push(Status::fail(
                "bad-numberOfHMetrics",
                &format!(
                    "The OpenType spec recommends at https://learn.microsoft.com/en-us/typography/opentype/spec/recom#hhea-table that hhea.numberOfHMetrics be set to 3 but this font has {number_of_h_metrics} instead.\nPlease read https://github.com/fonttools/fonttools/issues/3014 to decide whether this makes sense for your font.",
                ),
            ));
        }
        if !panose_is_monospaced(panose) {
            let family_type = panose[0];
            let advise = panose_expected(family_type);
            problems.push(Status::fail(
                "mono-bad-panose",
                &format!("The PANOSE numbers are incorrect for a monospaced font. {advise}"),
            ))
        }

        let num_glyphs = font.glyph_count;
        let metrics = font.font().hmtx()?;
        let unusually_spaced_glyphs: Vec<_> = metrics
            .h_metrics()
            .iter()
            .enumerate()
            .filter(|(gid, _x)| {
                let glyphname = font.glyph_name_for_id_synthesise(GlyphId::new(*gid as u16));
                *gid > 0 && glyphname != ".null" && glyphname != "NULL"
            })
            .filter(|(_gid, metric)| {
                metric.advance() != 0 && metric.advance() != statistics.most_common_width
            })
            .collect();
        let unusual_count = unusually_spaced_glyphs.len();
        let outliers_ratio = unusual_count as f32 / num_glyphs as f32 * 100f32;
        if outliers_ratio > 0.0 {
            problems.push(Status::fail(
                "mono-outliers",
                &format!(
                    "Font is monospaced but {unusual_count} glyphs ({outliers_ratio:.2}%) have a different width. You should check the widths of: {}",
                    bullet_list(context, unusually_spaced_glyphs.iter().map(|(gid, metric)| {
                        let glyphname = font.glyph_name_for_id_synthesise(GlyphId::new(*gid as u16));
                        format!("{} ({}), width: {}", glyphname, gid, metric.advance())
                    }))
                ),
            ));
        } else if post_isfixedpitch != 0 {
            problems.push(Status::fail(
                "mono-bad-post-isFixedPitch",
               &format!("On monospaced fonts, the value of post.isFixedPitch must be set to a non-zero value (meaning 'fixed width monospaced'), but got {post_isfixedpitch} instead.")
            ));
        }
    } else {
        // Not monospaced
        if post_isfixedpitch != 0 {
            problems.push(Status::fail(
                "bad-post-isFixedPitch",
                &format!("On non-monospaced fonts, the value of post.isFixedPitch must be set to a zero value (meaning 'not monospaced'), but got {post_isfixedpitch} instead.")
            ));
        }
        if panose[3] == 9 {
            // Proportion=Monospaced
            problems.push(Status::fail(
                "bad-panose",
                "On non-monospaced fonts, the OS/2.panose.bProportion value can be set to any value except 9 (proportion: monospaced) which is the bad value we got in this font."
            ));
        }
    }

    return_result(problems)
}

fn panose_is_monospaced(panose: &[u8]) -> bool {
    (panose[0] == 2 && panose[3] == 9)
        || (panose[0] == 3 && panose[3] == 3)
        || (panose[0] == 5 && panose[3] == 3)
}

fn panose_expected(family_type: u8) -> String {
    if family_type == 2 {
        // Latin Text
        return "Please set PANOSE Proportion to 9 (monospaced)".to_string();
    }
    if family_type == 3 || family_type == 5 {
        // Latin Hand Written or Latin Symbol
        return "Please set PANOSE Spacing to 3 (monospaced)".to_string();
    }
    "".to_string() // No advice for other types
}
struct GlyphMetricsStats {
    // At least 80% of encoded ASCII glyphs have the same width
    seems_monospaced: bool,
    // Largest advance width in the font
    width_max: u16,
    // Most common width
    most_common_width: u16,
}

fn most_common<I>(iter: impl Iterator<Item = I>) -> Option<(I, usize)>
where
    I: Eq,
    I: std::hash::Hash,
{
    let mut map = HashMap::new();
    for item in iter {
        *map.entry(item).or_insert(0) += 1;
    }
    map.into_iter().max_by_key(|(_, count)| *count)
}

fn glyph_metrics_stats(f: &TestFont) -> Result<GlyphMetricsStats, ReadError> {
    let metrics = f.font().hmtx()?;
    let ascii_glyph_ids = (32u32..127)
        .flat_map(|ch| f.font().charmap().map(ch))
        .collect::<Vec<_>>();
    let all_widths = metrics
        .h_metrics()
        .iter()
        .map(|x| x.advance())
        .filter(|x| *x != 0);
    let width_max = all_widths.clone().max().unwrap_or(0);
    let (most_common_width, _most_common_count) = most_common(all_widths).unwrap_or((0, 0));
    if ascii_glyph_ids.len() > 76 {
        // More than 80% of ASCII glyphs are present
        let ascii_widths = ascii_glyph_ids
            .iter()
            .flat_map(|id| metrics.h_metrics().get(id.to_u16() as usize))
            .map(|l| l.advance())
            .filter(|x| *x != 0);
        let ascii_widths_count = ascii_widths.clone().count() as f32;
        let (_most_common_ascii_width, most_common_ascii_count) =
            most_common(ascii_widths).unwrap_or((0, 0));
        let seems_monospaced = most_common_ascii_count as f32 > ascii_widths_count * 0.8;
        return Ok(GlyphMetricsStats {
            seems_monospaced,
            width_max,
            most_common_width,
        });
    }

    let mut widths = HashSet::new();
    for codepoint in f.codepoints() {
        #[allow(clippy::unwrap_used)] // We know it's mapped!
        let glyphid = f.font().charmap().map(codepoint).unwrap();
        // Skip separators, control and GDEF marks
        if char::from_u32(codepoint)
            .map(|c| {
                matches!(
                    c.general_category(),
                    GeneralCategory::LineSeparator
                        | GeneralCategory::ParagraphSeparator
                        | GeneralCategory::Control
                )
            })
            .unwrap_or(false)
            || f.gdef_class(glyphid) == GlyphClassDef::Mark
        {
            continue;
        }
        if let Some(width) = metrics
            .h_metrics()
            .get(glyphid.to_u16() as usize)
            .map(|l| l.advance())
        {
            if width != 0 {
                widths.insert(width);
            }
        }
    }

    Ok(GlyphMetricsStats {
        seems_monospaced: widths.len() <= 2,
        width_max,
        most_common_width,
    })
}
