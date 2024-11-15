use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;
use skrifa::{
    prelude::{LocationRef, Size},
    MetadataProvider,
};

#[check(
    id = "typoascender_exceeds_Agrave",
    rationale = "
        MacOS uses OS/2.sTypoAscender/Descender values to determine the line height
        of a font. If the sTypoAscender value is smaller than the maximum height of
        the uppercase /Agrave, the font’s sTypoAscender value is ignored, and a very
        tall line height is used instead.

        This happens on a per-font, per-style basis, so it’s possible for a font to
        have a good sTypoAscender value in one style but not in another. This can
        lead to inconsistent line heights across a typeface family.

        So, it is important to ensure that the sTypoAscender value is greater than
        the maximum height of the uppercase /Agrave in all styles of a type family.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3170",
    title = "Checking that the typoAscender exceeds the yMax of the /Agrave.",
    metadata = "{\"experimental\": \"since 2024/Jul/17\"}"
)]
fn typoascender_exceeds_agrave(f: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    let os2 = font
        .font()
        .os2()
        .map_err(|_| CheckError::Error("OS/2 table not found".to_string()))?;
    let agrave = font
        .font()
        .charmap()
        .map(0x00C0u32)
        .ok_or(CheckError::skip(
            "lacks-Agrave",
            "Font file lacks the /Agrave, so it can’t be compared with typoAscender",
        ))?;
    let metrics = font
        .font()
        .glyph_metrics(Size::unscaled(), LocationRef::new(&[]))
        .bounds(agrave)
        .ok_or(CheckError::Error(
            "Error getting bounds of Agrave (maybe it's empty?)".to_string(),
        ))?;
    let typo_ascender = os2.s_typo_ascender();
    Ok(if (typo_ascender as f32) < metrics.y_max {
        Status::just_one_warn(
            "typoAscender",
            &format!(
                "OS/2.sTypoAscender value should be greater than {}, but got {} instead",
                metrics.y_max, typo_ascender
            ),
        )
    } else {
        Status::just_one_pass()
    })
}
