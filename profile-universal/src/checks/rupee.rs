use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};

const INDIC_DETECTION_CODEPOINTS: [u32; 11] = [
    0x0988, // Bengali
    0x0908, // Devanagari
    0x0A88, // Gujarati
    0x0A08, // Gurmukhi
    0x0D08, // Kannada
    0x0B08, // Malayalam
    0xABC8, // Meetei Mayek
    0x1C58, // OlChiki
    0x0B08, // Oriya
    0x0B88, // Tamil
    0x0C08, // Telugu
];

#[check(
    id = "rupee",
    rationale = "
        Per Bureau of Indian Standards every font supporting one of the
        official Indian languages needs to include Unicode Character
        “₹” (U+20B9) Indian Rupee Sign.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/2967",
    title = "Ensure indic fonts have the Indian Rupee Sign glyph."
)]
fn rupee(f: &Testable, context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    let codepoints = font.codepoints(Some(context));
    if codepoints.contains(&0x20B9) {
        return Ok(Status::just_one_pass());
    }
    let is_indic = INDIC_DETECTION_CODEPOINTS
        .iter()
        .any(|cp| codepoints.contains(cp));
    if is_indic {
        return Ok(Status::just_one_fail(
            "missing-rupee",
            "Font appears to be an Indic font but is missing the Indian Rupee Sign glyph. Please add a glyph for Indian Rupee Sign (₹) at codepoint U+20B9.",
        ));
    } else {
        return Ok(Status::just_one_warn(
            "missing-rupee",
            "Font is missing the Indian Rupee Sign glyph. Please add a glyph for Indian Rupee Sign (₹) at codepoint U+20B9.",
        ));
    }
}
