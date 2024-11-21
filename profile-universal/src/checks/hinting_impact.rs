use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert, TestFont};
use read_fonts::ReadError;
use skrifa::{FontRef, Tag};
use write_fonts::{
    // from_obj::{FromObjRef, FromTableRef},
    // tables::glyf::Glyph,
    FontBuilder,
};

fn is_hinted(font: &TestFont) -> bool {
    if font.has_table(b"fpgm") || font.has_table(b"prep") || font.has_table(b"cvt ") {
        return true;
    }
    // We could (a) check for glyph-level hints, or (b) check inside CFF
    // table here, but I can't be bothered until someone demonstrates
    // that this is needed.
    false
}

fn dehinted(font: &FontRef) -> Result<Vec<u8>, ReadError> {
    let mut new_font = FontBuilder::new();
    for table in font.table_directory.table_records() {
        let tag = table.tag.get();
        if tag == Tag::new(b"fpgm") || tag == Tag::new(b"prep") || tag == Tag::new(b"cvt ") {
            continue;
        }
        if tag == Tag::new(b"glyf") {
            // https://github.com/googlefonts/fontations/issues/1253
            // let glyf: Glyf = font.glyf()?;
            // let loca = font.loca(None)?;
            // let glyph_count: u32 = font.maxp()?.num_glyphs().into();
            // let mut owned_glyphs: Vec<Glyph> = (0..glyph_count)
            //     .map(GlyphId::from)
            //     .flat_map(|gid| loca.get_glyf(gid, &glyf))
            //     .flatten()
            //     .map(|g| Glyph::from_table_ref(&g))
            //     .collect();
            // for glyph in owned_glyphs.iter_mut() {
            //     if let Glyph::Simple(ref mut simple) = glyph {
            //          // Actually there's nothing intelligent I can do here.
            //     }
            // }

            log::warn!("glyf table dehinting not implemented");
            #[allow(clippy::unwrap_used)]
            new_font.add_raw(tag, font.table_data(tag).unwrap());
            continue;
        }
        if let Some(table) = font.table_data(tag) {
            new_font.add_raw(tag, table);
        }
    }
    Ok(new_font.build())
}

#[check(
    id = "hinting_impact",
    rationale = "
        This check is merely informative, displaying an useful comparison of filesizes
        of hinted versus unhinted font files.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Show hinting filesize impact."
)]
fn hinting_impact(f: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    skip!(!is_hinted(&font), "not-hinted", "Font is not hinted");
    let hinted_size = f.contents.len();
    let dehinted = dehinted(&font.font())?;
    let dehinted_size: usize = dehinted.len();
    let increase = hinted_size - dehinted_size;
    let change = ((hinted_size as f32) / (dehinted_size as f32) - 1.0) * 100.0;

    Ok(Status::just_one_info(
        "size-impact",
        &format!(
            "Hinting filesize impact:

 |               | {}     |
 |:------------- | ---------------:|
 | Dehinted Size | {} |
 | Hinted Size   | {}   |
 | Increase      | {}      |
 | Change        | {:.1} %  |",
            f.basename().unwrap_or("Font file".to_string()),
            dehinted_size,
            hinted_size,
            increase,
            change,
        ),
    ))
}
