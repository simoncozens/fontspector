use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use read_fonts::{
    tables::glyf::{Anchor, Glyph},
    TableProvider,
};
use skrifa::GlyphId;
use std::collections::HashSet;

#[check(
    id = "opentype/glyf_non_transformed_duplicate_components",
    rationale = "
        There have been cases in which fonts had faulty double quote marks, with each
        of them containing two single quote marks as components with the same
        x, y coordinates which makes them visually look like single quote marks.

        This check ensures that glyphs do not contain duplicate components
        which have the same x,y coordinates.
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/2709",
    title = "Check glyphs do not have duplicate components which have the same x,y coordinates."
)]
fn glyf_non_transformed_duplicate_components(
    t: &Testable,
    context: &Context,
) -> CheckFnResult {
    let ttf = testfont!(t);
    let font = ttf.font();
    skip!(!ttf.has_table(b"glyf"), "no-glyf", "No glyf table");
    let glyf = font.glyf()?;
    let loca = font.loca(None)?;
    let mut messages = vec![];
    for gid in 0..font.maxp()?.num_glyphs() {
        let gid = GlyphId::new(gid.into());
        if let Some(Glyph::Composite(glyph)) = loca.get_glyf(gid, &glyf)? {
            let mut components = HashSet::new();
            for component in glyph.components() {
                if let Anchor::Offset { x, y } = component.anchor {
                    if !components.insert((component.glyph, x, y)) {
                        messages.push(format!(
                            "{}: duplicate component {} at {},{}",
                            ttf.glyph_name_for_id_synthesise(gid),
                            ttf.glyph_name_for_id_synthesise(component.glyph),
                            x,
                            y
                        ));
                    }
                }
            }
        }
    }
    if messages.is_empty() {
        Ok(Status::just_one_pass())
    } else {
        Ok(
            Status::just_one_fail("found-duplicates", 
                &format!("The following glyphs have duplicate components which have the same x,y coordinates.\n\n{}",
                    bullet_list(context, messages))
            )
        )
    }
}
