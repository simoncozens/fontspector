use fontspector_checkapi::{
    constants::{INDIC_SCRIPT_TAGS, USE_SCRIPT_TAGS},
    prelude::*,
    skip, testfont, FileTypeConvert, TestFont,
};
use hashbrown::{HashMap, HashSet};
use read_fonts::{
    tables::{gdef::GlyphClassDef, gpos::PositionSubtables},
    TableProvider,
};
use skrifa::{GlyphId, MetadataProvider};

fn is_complex_shaper_font(f: &TestFont) -> Option<bool> {
    for script_list in [
        f.font().gsub().ok()?.script_list().ok()?,
        f.font().gpos().ok()?.script_list().ok()?,
    ]
    .iter()
    {
        let complex_script = script_list
            .script_records()
            .iter()
            .map(|s| s.script_tag().to_string())
            .any(|tag| {
                USE_SCRIPT_TAGS.contains(&tag.as_str())
                    || INDIC_SCRIPT_TAGS.contains(&tag.as_str())
                    || tag == "khmr"
                    || tag == "mymr"
            });
        if complex_script {
            return Some(true);
        }
    }
    Some(false)
}

type AttachmentMap = HashMap<GlyphId, Vec<GlyphId>>;

fn find_all_attachments(f: &TestFont) -> Result<(HashSet<GlyphId>, AttachmentMap), CheckError> {
    let attachment_rules = f
        .font()
        .gpos()?
        .lookup_list()?
        .lookups()
        .iter()
        .flatten()
        .flat_map(|l| l.subtables())
        .filter_map(|s| match s {
            PositionSubtables::MarkToBase(p) => Some(p),
            _ => None,
        })
        .flat_map(|p| p.iter())
        .flatten();
    let mut does_attach = HashSet::new();
    let mut attachments = HashMap::new();
    for attachment in attachment_rules {
        for base in attachment.base_coverage()?.iter() {
            for mark in attachment.mark_coverage()?.iter() {
                attachments
                    .entry(base.into())
                    .or_insert_with(Vec::new)
                    .push(mark.into());
                does_attach.insert(mark.into());
            }
        }
    }
    Ok((does_attach, attachments))
}

#[check(
    id = "dotted_circle",
    rationale = "
        
        The dotted circle character (U+25CC) is inserted by shaping engines before
        mark glyphs which do not have an associated base, especially in the context
        of broken syllabic clusters.

        For fonts containing combining marks, it is recommended that the dotted circle
        character be included so that these isolated marks can be displayed properly;
        for fonts supporting complex scripts, this should be considered mandatory.

        Additionally, when a dotted circle glyph is present, it should be able to
        display all marks correctly, meaning that it should contain anchors for all
        attaching marks.

        A fontmake filter can be used to automatically add a dotted_circle to a font:

        fontmake --filter 'DottedCircleFilter(pre=True)' --filter '...'
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3600",
    title = "Ensure dotted circle glyph is present and can attach marks."
)]
fn dotted_circle(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    let mappings = f.font().charmap().mappings().collect::<Vec<_>>();
    let mapped: HashSet<GlyphId> = mappings.iter().map(|(_u, gid)| *gid).collect();
    let hmtx = f.font().hmtx()?;
    let nonspacing_mark_glyphs = f
        .all_glyphs()
        .filter(|g| f.gdef_class(*g) == GlyphClassDef::Mark)
        .filter(|g| mapped.contains(g))
        .filter(|g| hmtx.advance(*g).unwrap_or(0) == 0)
        .collect::<Vec<_>>();
    if nonspacing_mark_glyphs.is_empty() {
        skip!("no-nonspacing-marks", "Font has no nonspacing mark glyphs.");
    }
    let dotted_circle_glyph = mappings.iter().find(|(u, _g)| u == &0x25CC);
    if let Some((_u, dc_gid)) = dotted_circle_glyph {
        let (does_attach, attachments) = find_all_attachments(&f)?;
        let mut unattached = vec![];
        let dc_attachments = attachments.get(dc_gid);
        for g in nonspacing_mark_glyphs {
            if does_attach.contains(&g) && !dc_attachments.is_some_and(|o| o.contains(&g)) {
                unattached.push(g);
            }
        }
        if !unattached.is_empty() {
            problems.push(Status::fail(
                "unattached-dotted-circle-marks",
                &format!(
                    "The following glyphs could not be attached to the dotted circle glyph:\n\n{}",
                    bullet_list(
                        context,
                        unattached
                            .into_iter()
                            .map(|g| f.glyph_name_for_id_synthesise(g))
                    )
                ),
            ));
        }
    } else if is_complex_shaper_font(&f).unwrap_or(false) {
        problems.push(Status::fail(
            "missing-dotted-circle-complex",
            "No dotted circle glyph present and font uses a complex shaper",
        ));
    } else {
        problems.push(Status::warn(
            "missing-dotted-circle",
            "No dotted circle glyph present",
        ));
    }
    return_result(problems)
}
