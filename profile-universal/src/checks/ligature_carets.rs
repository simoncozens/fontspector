use std::collections::HashSet;

use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use read_fonts::{tables::gdef::GlyphClassDef, TableProvider};
use skrifa::GlyphId;

#[check(
    id = "ligature_carets",
    rationale = "
        
        All ligatures in a font must have corresponding caret (text cursor) positions
        defined in the GDEF table, otherwhise, users may experience issues with
        caret rendering.

        If using GlyphsApp or UFOs, ligature carets can be defined as anchors with
        names starting with `caret_`. These can be compiled with fontmake as of
        version v2.4.0.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/1225",
    title = "Are there caret positions declared for every ligature?"
)]
fn ligature_carets(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let ligature_glyphs = f
        .all_glyphs()
        .filter(|g| f.gdef_class(*g) == GlyphClassDef::Ligature)
        .collect::<HashSet<_>>();
    skip!(
        ligature_glyphs.is_empty(),
        "no-ligatures",
        "No ligature glyphs found."
    );
    let (missing, has_carets): (Vec<GlyphId>, bool) =
        if let Some(ligcaretlist) = f.font().gdef()?.lig_caret_list() {
            let ligcaretlist = ligcaretlist?;
            let has_carets = ligcaretlist.lig_glyph_count() > 0;
            let collect = ligcaretlist
                .coverage()?
                .iter()
                .map(|x| x.into())
                .collect::<HashSet<_>>();
            (
                ligature_glyphs.difference(&collect).copied().collect(),
                has_carets,
            )
        } else {
            (ligature_glyphs.iter().copied().collect(), false)
        };
    if !has_carets {
        return Ok(Status::just_one_warn(
            "lacks-caret-pos",
            "This font lacks caret position values for ligature glyphs on its GDEF table.",
        ));
    }
    if !missing.is_empty() {
        return Ok(Status::just_one_warn(
            "incomplete-caret-pos-data",
            &format!(
                "This font lacks caret positioning values for these ligature glyphs:\n\t- {}\n\n",
                bullet_list(
                    context,
                    missing.iter().map(|g| f.glyph_name_for_id_synthesise(*g))
                )
            ),
        ));
    }
    Ok(Status::just_one_pass())
}
