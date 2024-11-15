use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use skrifa::{outline::OutlinePen, setting::VariationSetting, GlyphId, MetadataProvider};

#[derive(Default)]
struct HasInkPen(bool);
impl HasInkPen {
    fn has_ink(&self) -> bool {
        self.0
    }
}
impl OutlinePen for HasInkPen {
    fn move_to(&mut self, _x: f32, _y: f32) {}

    fn line_to(&mut self, _x: f32, _y: f32) {
        self.0 = true;
    }

    fn quad_to(&mut self, _cx0: f32, _cy0: f32, _x: f32, _y: f32) {
        self.0 = true;
    }

    fn curve_to(&mut self, _cx0: f32, _cy0: f32, _cx1: f32, _cy1: f32, _x: f32, _y: f32) {
        self.0 = true;
    }

    fn close(&mut self) {}
}

#[check(
    id="mandatory_glyphs",
    rationale="
        The OpenType specification v1.8.2 recommends that the first glyph is the
        '.notdef' glyph without a codepoint assigned and with a drawing:

        The .notdef glyph is very important for providing the user feedback
        that a glyph is not found in the font. This glyph should not be left
        without an outline as the user will only see what looks like a space
        if a glyph is missing and not be aware of the active font’s limitation.

        https://docs.microsoft.com/en-us/typography/opentype/spec/recom#glyph-0-the-notdef-glyph

        Pre-v1.8, it was recommended that fonts should also contain 'space', 'CR'
        and '.null' glyphs. This might have been relevant for MacOS 9 applications.
    ",
    title="Font contains '.notdef' as its first glyph?",
    proposal="https://github.com/fonttools/fontbakery/issues/4829",  // legacy check
)]
fn mandatory_glyphs(f: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    let mut problems = vec![];
    let gid_0 = GlyphId::new(0);
    if font.glyph_name_for_id(gid_0) != Some(".notdef".to_string()) {
        // Is notdef somewhere else?!
        if font
            .all_glyphs()
            .any(|g| font.glyph_name_for_id(g) == Some(".notdef".to_string()))
        {
            problems.push(Status::warn(
                "notdef-not-first",
                "The '.notdef' glyph should be the font's first glyph.",
            ))
        } else {
            problems.push(Status::warn(
                "notdef-not-found",
                "Font should contain the '.notdef' glyph.",
            ))
        }
    }
    if let Some(cp) = font.font().charmap().mappings().find(|m| m.1 == gid_0) {
        problems.push(Status::warn(
            "notdef-has-codepoint",
            &format!("The '.notdef' glyph should not have a Unicode codepoint value assigned, but has 0x{:04X}.", cp.0)
        ))
    }
    let mut pen = HasInkPen::default();
    let default: Vec<VariationSetting> = vec![];
    font.draw_glyph(gid_0, &mut pen, default)?;
    if !pen.has_ink() {
        problems.push(Status::fail(
            "notdef-is-blank",
            "The '.notdef' glyph should contain a drawing, but it is blank.",
        ))
    }

    return_result(problems)
}
