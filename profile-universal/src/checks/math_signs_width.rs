use std::collections::{HashMap, HashSet};

use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use itertools::Itertools;
use read_fonts::TableProvider;
use skrifa::{GlyphId, MetadataProvider};

const COMMON_WIDTH_MATH_CHARS: [char; 314] = [
    '+', '<', '=', '>', '¬', '±', '×', '÷', '∈', '∉', '∋', '∌', '−', '∓', '∔', '∝', '∟', '∠', '∡',
    '∢', '∷', '∸', '∹', '∺', '∻', '∼', '∽', '∾', '∿', '≁', '≂', '≃', '≄', '≅', '≆', '≇', '≈', '≉',
    '≊', '≋', '≌', '≍', '≎', '≏', '≐', '≑', '≒', '≓', '≖', '≗', '≘', '≙', '≚', '≛', '≜', '≝', '≞',
    '≟', '≠', '≡', '≢', '≣', '≤', '≥', '≦', '≧', '≨', '≩', '≭', '≮', '≯', '≰', '≱', '≲', '≳', '≴',
    '≵', '≶', '≷', '≸', '≹', '≺', '≻', '≼', '≽', '≾', '≿', '⊀', '⊁', '⊂', '⊃', '⊄', '⊅', '⊆', '⊇',
    '⊈', '⊉', '⊊', '⊋', '⊏', '⊐', '⊑', '⊒', '⊢', '⊣', '⊤', '⊥', '⊨', '⊰', '⊱', '⊲', '⊳', '⊴', '⊵',
    '⊹', '⊾', '⋇', '⋍', '⋐', '⋑', '⋕', '⋖', '⋗', '⋚', '⋛', '⋜', '⋝', '⋞', '⋟', '⋠', '⋡', '⋢', '⋣',
    '⋤', '⋥', '⋦', '⋧', '⋨', '⋩', '⋳', '⋵', '⋶', '⋸', '⋹', '⋻', '⋽', '⟀', '⟃', '⟄', '⟓', '⟔', '⥶',
    '⥸', '⥹', '⥻', '⥾', '⥿', '⦓', '⦔', '⦕', '⦖', '⦛', '⦜', '⦝', '⦞', '⦟', '⦠', '⦡', '⦢', '⦣', '⦤',
    '⦥', '⦨', '⦩', '⦪', '⦫', '⧣', '⧤', '⧥', '⧺', '⧻', '⨢', '⨣', '⨤', '⨥', '⨦', '⨧', '⨨', '⨩', '⨪',
    '⨫', '⨬', '⨳', '⩦', '⩧', '⩨', '⩩', '⩪', '⩫', '⩬', '⩭', '⩮', '⩯', '⩰', '⩱', '⩲', '⩳', '⩷', '⩸',
    '⩹', '⩺', '⩻', '⩼', '⩽', '⩾', '⩿', '⪀', '⪁', '⪂', '⪃', '⪄', '⪅', '⪆', '⪇', '⪈', '⪉', '⪊', '⪋',
    '⪌', '⪍', '⪎', '⪏', '⪐', '⪑', '⪒', '⪓', '⪔', '⪕', '⪖', '⪗', '⪘', '⪙', '⪚', '⪛', '⪜', '⪝', '⪞',
    '⪟', '⪠', '⪡', '⪢', '⪦', '⪧', '⪨', '⪩', '⪪', '⪫', '⪬', '⪭', '⪮', '⪯', '⪰', '⪱', '⪲', '⪳', '⪴',
    '⪵', '⪶', '⪷', '⪸', '⪹', '⪺', '⪽', '⪾', '⪿', '⫀', '⫁', '⫂', '⫃', '⫄', '⫅', '⫆', '⫇', '⫈', '⫉',
    '⫊', '⫋', '⫌', '⫏', '⫐', '⫑', '⫒', '⫓', '⫔', '⫕', '⫖', '⫟', '⫠', '⫡', '⫢', '⫤', '⫦', '⫧', '⫨',
    '⫩', '⫪', '⫫', '⫳', '⫴', '⫵', '⫶', '⫹', '⫺', '〒',
];

#[check(
    id = "math_signs_width",
    rationale = "
        It is a common practice to have math signs sharing the same width
        (preferably the same width as tabular figures accross the entire font family).

        This probably comes from the will to avoid additional tabular math signs
        knowing that their design can easily share the same width.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3832",
    title = "Check math signs have the same width."
)]
fn math_signs_width(f: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    let charmap = font.font().charmap();
    let hmtx = font.font().hmtx()?;
    let mut widths: HashMap<u16, HashSet<GlyphId>> = HashMap::new();
    for (glyph, width) in COMMON_WIDTH_MATH_CHARS
        .iter()
        .flat_map(|c| charmap.map(*c as u32))
        .map(|gid| (gid, hmtx.advance(gid).unwrap_or(0)))
    {
        widths.entry(width).or_default().insert(glyph);
    }

    let most_common_width = widths
        .iter()
        .max_by_key(|(_, glyphs)| glyphs.len())
        .map(|(width, _)| *width);
    if widths.len() == 1 {
        return Ok(Status::just_one_pass());
    }
    let num_glyphs = widths.values().map(|g| g.len()).sum::<usize>();
    if let Some(width) = most_common_width {
        let summary = widths
            .into_iter()
            .filter(|(w, _)| *w != width)
            .map(|(w, glyphs)| {
                format!(
                    "width={}: {}",
                    w,
                    glyphs
                        .iter()
                        .map(|gid| font.glyph_name_for_id_synthesise(*gid))
                        .join(", ")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        return Ok(Status::just_one_warn(
            "width-outliers",
            &format!("The most common width is {} among a set of {}  math glyphs.\nThe following math glyphs have a different width, though:\n{}"
            , width, num_glyphs, summary)
        ));
    }
    // No most common
    return Ok(Status::just_one_pass());
}
