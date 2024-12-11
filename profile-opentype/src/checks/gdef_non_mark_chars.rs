use crate::checks::gdef::is_nonspacing_mark;
use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;
use skrifa::{GlyphId16, MetadataProvider};

fn swaption<T, U>(a: T, b: Option<U>) -> Option<(T, U)> {
    b.map(|b| (a, b))
}

#[check(
    id = "opentype/gdef_non_mark_chars",
    rationale = "
        Glyphs in the GDEF mark glyph class become non-spacing and may be repositioned
        if they have mark anchors.

        Only combining mark glyphs should be in that class. Any non-mark glyph
        must not be in that class, in particular spacing glyphs.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/2877",
    title = "Check GDEF mark glyph class doesn't have characters that are not marks."
)]
fn gdef_non_mark_chars(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let gdef = f
        .font()
        .gdef()
        .map_err(|_| CheckError::skip("no-gdef", "GDEF table unreadable or not present"))?;
    let glyph_classdef = gdef.glyph_class_def().ok_or_else(|| {
        CheckError::skip("no-glyph-class-def", "GDEF table has no GlyphClassDef")
    })??;
    let codepoints = f.codepoints(Some(context));
    let non_mark_gids = codepoints
        .iter()
        .flat_map(|cp| char::from_u32(*cp))
        .filter(|&cp| !is_nonspacing_mark(cp))
        .flat_map(|cp| swaption(cp, f.font().charmap().map(cp)))
        .flat_map(|(cp, gid)| swaption(cp, GlyphId16::try_from(gid).ok()));
    let non_mark_gids_in_mark = non_mark_gids.filter(|(_cp, gid)| glyph_classdef.get(*gid) == 3);
    if non_mark_gids_in_mark.clone().count() > 0 {
        return Ok(Status::just_one_warn(
            "non-mark-chars",
            &format!(
                "The following non-mark characters should not be in the GDEF mark glyph class:\n{}",
                bullet_list(
                    context,
                    non_mark_gids_in_mark.map(|(cp, gid)| format!(
                        "U+{:04X} ({})",
                        cp as u32,
                        f.glyph_name_for_id_synthesise(gid)
                    ))
                ),
            ),
        ));
    }
    Ok(Status::just_one_pass())
}
