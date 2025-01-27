use std::collections::HashMap;

use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::tables::gdef::GlyphClassDef;
use read_fonts::TableProvider;
use skrifa::{GlyphId, MetadataProvider};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

fn is_space(c: char) -> bool {
    matches!(
        c.general_category(),
        GeneralCategory::SpaceSeparator
            | GeneralCategory::LineSeparator
            | GeneralCategory::ParagraphSeparator
            | GeneralCategory::Format
            | GeneralCategory::NonspacingMark
            | GeneralCategory::Control
    )
}

#[check(
    id = "base_has_width",
    rationale = "Base characters should have non-zero advance width.",
    proposal = "Rod on chat",
    title = "Check base characters have non-zero advance width."
)]
fn base_has_width(f: &Testable, context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    let hmtx = font.font().hmtx()?;
    let mut problems = vec![];
    let reverse_charmap: HashMap<_, _> = font
        .font()
        .charmap()
        .mappings()
        .map(|(c, g)| (g, c))
        .collect();
    for (gid, metric) in hmtx.h_metrics().iter().enumerate() {
        let gid = GlyphId::new(gid as u32);
        if metric.advance() == 0 && font.gdef_class(gid) != GlyphClassDef::Mark {
            let codepoint = reverse_charmap.get(&gid);
            if codepoint == Some(&0) || codepoint.is_none() {
                continue;
            }
            if codepoint
                .and_then(|c| char::from_u32(*c))
                .is_some_and(is_space)
            {
                continue;
            }
            #[allow(clippy::unwrap_used)]
            let name = font.glyph_name_for_id_synthesise(gid);
            if name == "NULL" {
                continue;
            }
            problems.push(format!("{} ({:?})", name, codepoint));
        }
    }
    if problems.is_empty() {
        Ok(Status::just_one_pass())
    } else {
        Ok(Status::just_one_fail(
            "zero-width-bases",
            &format!(
                "The following glyphs had zero advance width:\n{}",
                bullet_list(context, problems),
            ),
        ))
    }
}
