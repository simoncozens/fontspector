use std::collections::HashSet;

use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use read_fonts::TableProvider;

const MINIMUM_BRIGHTNESS: f32 = 0.1 * 256.0;

#[check(
    id = "color_cpal_brightness",
    rationale = "
         Layers of a COLRv0 font should not be too dark or too bright. When layer colors
        are set explicitly, they can't be changed and they may turn out illegible
        against dark or bright backgrounds.

        While traditional color-less fonts can be colored in design apps or CSS, a
        black color definition in a COLRv0 font actually means that that layer will be
        rendered in black regardless of the background color. This leads to text
        becoming invisible against a dark background, for instance when using a dark
        theme in a web browser or operating system.

        This check ensures that layer colors are at least 10% bright and at most 90%
        bright, when not already set to the current color (0xFFFF).
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/3908",
    title = "Color layers should have a minimum brightness."
)]
fn color_cpal_brightness(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut dark_glyphs = HashSet::new();
    skip!(
        !f.has_table(b"CPAL"),
        "no-cpal",
        "Font does not contain a CPAL table."
    );
    skip!(
        f.has_table(b"COLR") && f.font().colr()?.version() != 0,
        "colr-v1",
        "Font contains a COLR table, but it's not version 0."
    );
    let color = f
        .font()
        .cpal()?
        .color_records_array()
        .transpose()?
        .ok_or_else(|| {
            CheckError::skip(
                "no-color-records",
                "Font does not contain any color records.",
            )
        })?;
    let layers = f
        .font()
        .colr()?
        .layer_records()
        .transpose()?
        .ok_or_else(|| {
            CheckError::skip(
                "no-layer-records",
                "Font does not contain any layer records.",
            )
        })?;
    for layer in layers {
        if layer.palette_index() == 0xFFFF {
            continue;
        }
        let color = color
            .get(layer.palette_index() as usize)
            .ok_or_else(|| CheckError::Error("invalid-palette-index".to_string()))?;

        let brightness = ((color.red as f32 * 299.0)
            + (color.green as f32 * 587.0)
            + (color.blue as f32 * 114.0))
            / 1000.0;
        if !(MINIMUM_BRIGHTNESS..=256.0 - MINIMUM_BRIGHTNESS).contains(&brightness) {
            dark_glyphs.insert(layer.glyph_id());
        }
    }
    if dark_glyphs.is_empty() {
        Ok(Status::just_one_pass())
    } else {
        Ok(Status::just_one_fail(
            "dark-colors",
            &format!(
                "These glyphs have layers with colors that are too dark or too bright: {}",
                bullet_list(
                    context,
                    dark_glyphs
                        .iter()
                        .map(|g| f.glyph_name_for_id_synthesise(*g))
                )
            ),
        ))
    }
}
