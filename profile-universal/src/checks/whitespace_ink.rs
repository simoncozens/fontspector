use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use skrifa::{outline::OutlinePen, setting::VariationSetting, MetadataProvider};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

const EXTRA_NON_DRAWING: [u32; 4] = [0x180E, 0x200B, 0x2060, 0xFEFF];
const BUT_NOT: [u32; 2] = [0xAD, 0x1680];

#[derive(Default)]
struct AnythingPen {
    anything: bool,
}
impl OutlinePen for AnythingPen {
    fn move_to(&mut self, _x: f32, _y: f32) {}
    fn line_to(&mut self, _x: f32, _y: f32) {
        self.anything = true;
    }
    fn curve_to(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _x3: f32, _y3: f32) {
        self.anything = true;
    }
    fn quad_to(&mut self, _cx0: f32, _cy0: f32, _x: f32, _y: f32) {
        self.anything = true;
    }
    fn close(&mut self) {}
}

#[check(
    id = "whitespace_ink",
    rationale = "
           This check ensures that certain whitespace glyphs are empty.
           Certain text layout engines will assume that these glyphs are empty,
           and will not draw them; if they were in fact not designed to be
           empty, the result will be text layout that is not as expected.
       ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Whitespace glyphs have ink?"
)]
fn whitespace_ink(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let inky = f
        .codepoints()
        .into_iter()
        .filter(|cp| {
            (EXTRA_NON_DRAWING.contains(cp)
                || (char::from_u32(*cp)
                    .map(|c| {
                        matches!(
                            c.general_category(),
                            GeneralCategory::SpaceSeparator | GeneralCategory::Format
                        )
                    })
                    .unwrap_or(false)))
                && !BUT_NOT.contains(cp)
        })
        .map(|cp| {
            #[allow(clippy::unwrap_used)]
            let gid = f.font().charmap().map(cp).unwrap();
            (cp, gid)
        })
        .filter(|(_cp, gid)| {
            let default: Vec<VariationSetting> = vec![];
            let mut anythingpen = AnythingPen::default();
            f.draw_glyph(*gid, &mut anythingpen, default)
                .ok()
                .and(if anythingpen.anything { Some(()) } else { None })
                .is_some()
        })
        .map(|(_cp, gid)| f.glyph_name_for_id_synthesise(gid))
        .collect::<Vec<_>>();
    if inky.is_empty() {
        Ok(Status::just_one_pass())
    } else {
        Ok(Status::just_one_fail(
            "has-ink",
            &format!(
                "The following glyphs have ink; they should be replaced by an empty glyph:\n{}",
                bullet_list(context, inky)
            ),
        ))
    }
}
