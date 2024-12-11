use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::tables::name::NameId;
use skrifa::MetadataProvider;

#[check(
    id = "render_own_name",
    title = "Ensure font can render its own name.",
    rationale = "
        A base expectation is that a font family's regular/default (400 roman) style
        can render its 'menu name' (nameID 1) in itself.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3159"
)]
fn render_own_name(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let name = f
        .font()
        .localized_strings(NameId::FAMILY_NAME)
        .english_or_first()
        .ok_or(CheckError::Error("Family name not found".to_string()))?;
    let codepoints = f.codepoints(Some(context));
    if name.chars().any(|c| !codepoints.contains(&(c as u32))) {
        Ok(Status::just_one_fail(
            "render-own-name",
            &format!(
                ".notdef glyphs were found when attempting to render {}",
                name.chars().collect::<String>()
            ),
        ))
    } else {
        Ok(Status::just_one_pass())
    }
}
