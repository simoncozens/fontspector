use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};

const ARABIC_SPACING_SYMBOLS: [u16; 17] = [
    0xFBB2, // Dot Above
    0xFBB3, // Dot Below
    0xFBB4, // Two Dots Above
    0xFBB5, // Two Dots Below
    0xFBB6, // Three Dots Above
    0xFBB7, // Three Dots Below
    0xFBB8, // Three Dots Pointing Downwards Above
    0xFBB9, // Three Dots Pointing Downwards Below
    0xFBBA, // Four Dots Above
    0xFBBB, // Four Dots Below
    0xFBBC, // Double Vertical Bar Below
    0xFBBD, // Two Dots Vertically Above
    0xFBBE, // Two Dots Vertically Below
    0xFBBF, // Ring
    0xFBC0, // Small Tah Above
    0xFBC1, // Small Tah Below
    0xFBC2, // Wasla Above
];

#[check(
    id = "com.google.fonts/check/arabic_spacing_symbols",
    title = "Check that Arabic spacing symbols U+FBB2–FBC1 aren't classified as marks.",
    rationale = "
        Unicode has a few spacing symbols representing Arabic dots and other marks,
        but they are purposefully not classified as marks.

        Many fonts mistakenly classify them as marks, making them unsuitable
        for their original purpose as stand-alone symbols to used in pedagogical
        contexts discussing Arabic consonantal marks.
    ",
    proposal = "https://github.com/googlefonts/fontbakery/issues/4295"
)]
fn arabic_spacing_symbols(t: &Testable, _context: &Context) -> CheckFnResult {
    let mut problems: Vec<Status> = vec![];
    let f = testfont!(t);
    let cmap = f
        .get_cmap()
        .map_err(|_| CheckError::Error("Font lacks a cmap table".to_string()))?;
    let gdef = f
        .get_gdef()
        .map_err(|_| CheckError::Error("Font lacks a gdef table".to_string()))?;

    let class_def = match gdef.glyph_class_def() {
        None => return return_result(problems),
        Some(d) => d.map_err(|e| CheckError::Error(format!("Some classDef error: {}", e)))?,
    };

    for codepoint in ARABIC_SPACING_SYMBOLS {
        let gid = cmap.map_codepoint(codepoint);
        if gid.is_some()
            && class_def.get(gid.ok_or(CheckError::Error("Failed to read gid".to_string()))?) == 3
        {
            problems.push(Status::fail(
                "gdef-mark",
                &format!(
                    "U+{:04X} is defined in GDEF as a mark (class 3).",
                    codepoint
                ),
            ));
        }
    }

    if problems.is_empty() {
        Ok(Status::just_one_pass())
    } else {
        return_result(problems)
    }
}
