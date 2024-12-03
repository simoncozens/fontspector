use std::collections::HashSet;

use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::{tables::cmap::CmapSubtable, TableProvider};

#[check(
    id = "cmap/format_12",
    rationale = "
        If a format 12 cmap table is used to address codepoints beyond the BMP,
        it should actually contain such codepoints. Additionally, it should also
        contain all characters mapped in the format 4 subtable.
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/3681",
    title = "Check that format 12 cmap subtables are correctly constituted."
)]
fn cmap_format_12(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let cmap = f.font().cmap()?;
    let format_4 = cmap
        .encoding_records()
        .iter()
        .flat_map(|x| x.subtable(cmap.offset_data()))
        .find(|x| x.format() == 4);
    let format_4_codepoints = if let Some(CmapSubtable::Format4(format_4)) = format_4 {
        format_4
            .iter()
            .map(|(cp, _glyph)| cp)
            .collect::<HashSet<_>>()
    } else {
        HashSet::new()
    };
    let mut skipped = true;
    let mut problems = vec![];
    for subtable in cmap
        .encoding_records()
        .iter()
        .flat_map(|x| x.subtable(cmap.offset_data()))
    {
        if let CmapSubtable::Format12(subtable) = subtable {
            skipped = false;
            if !subtable.iter().map(|(cp, _glyph)| cp).any(|cp| cp > 0x0FFF) {
                problems.push(Status::fail(
                    "pointless-format-12",
                "A format 12 subtable did not contain any codepoints beyond the Basic Multilingual Plane (BMP)"
                ))
            }
            let cmap12_codepoints: HashSet<_> = subtable.iter().map(|(cp, _glyph)| cp).collect();
            let unmapped = format_4_codepoints
                .difference(&cmap12_codepoints)
                .collect::<Vec<_>>();
            if !unmapped.is_empty() {
                problems.push(Status::warn(
                    "missing-format-4",
                    &format!(
                        "The format 12 subtable did not contain all codepoints from the format 4 subtable: {}",
                        bullet_list(context, unmapped)
                    )
                ))
            }
        }
    }
    if skipped {
        Ok(Status::just_one_skip(
            "no-format-12",
            "No format 12 subtable was found",
        ))
    } else {
        Ok(Status::just_one_pass())
    }
}
