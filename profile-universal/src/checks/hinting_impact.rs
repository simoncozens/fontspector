use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert, TestFont};
use read_fonts::{tables::glyf::Glyf, ReadError, TableProvider};
use skrifa::{FontRef, GlyphId, Tag};
use write_fonts::{
    from_obj::FromTableRef,
    tables::glyf::{GlyfLocaBuilder, Glyph},
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

fn dehinted(font: &FontRef) -> Result<Vec<u8>, CheckError> {
    let mut new_font = FontBuilder::new();
    let glyf_table_hinted = any_glyphs_have_instructions(font)?;
    for table in font.table_directory.table_records() {
        let tag = table.tag.get();
        if tag == Tag::new(b"fpgm") || tag == Tag::new(b"prep") || tag == Tag::new(b"cvt ") {
            continue;
        }
        if tag == Tag::new(b"glyf") && glyf_table_hinted {
            // https://github.com/googlefonts/fontations/issues/1253
            let glyf: Glyf = font.glyf()?;
            let loca = font.loca(None)?;
            let glyph_count: u32 = font.maxp()?.num_glyphs().into();
            let mut builder = GlyfLocaBuilder::new();
            let mut owned_glyphs: Vec<Glyph> = (0..glyph_count)
                .map(GlyphId::from)
                .flat_map(|gid| loca.get_glyf(gid, &glyf))
                .flatten()
                .map(|g| Glyph::from_table_ref(&g))
                .collect();
            for glyph in owned_glyphs.iter_mut() {
                if let Glyph::Simple(ref mut _simple) = glyph {
                    // Coming to a write-fonts near you soon!
                    log::warn!("TTF dehinting not yet implemented; upgrade write-fonts");
                    // simple.instructions = vec![];
                }
                builder.add_glyph(glyph)?;
            }
            let (glyf, loca, _loca_format) = builder.build();
            new_font.add_table(&glyf)?;
            new_font.add_table(&loca)?;
            continue;
        }
        if let Some(table) = font.table_data(tag) {
            new_font.add_raw(tag, table);
        }
    }
    Ok(new_font.build())
}

fn any_glyphs_have_instructions(font: &FontRef<'_>) -> Result<bool, ReadError> {
    let glyf: Glyf = font.glyf()?;
    let loca = font.loca(None)?;
    let glyph_count: u32 = font.maxp()?.num_glyphs().into();
    Ok((0..glyph_count)
        .map(GlyphId::from)
        .flat_map(|gid| loca.get_glyf(gid, &glyf))
        .flatten()
        .take(100) // Limit to 100 glyphs to avoid performance issues
        .any(|g| match g {
            read_fonts::tables::glyf::Glyph::Simple(simple) => !simple.instructions().is_empty(),
            _ => false,
        }))
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
    let increase = hinted_size as isize - dehinted_size as isize;
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
