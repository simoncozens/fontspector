use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};

#[check(
    id = "control_chars",
    rationale = "
        Use of some unacceptable control characters in the U+0000 - U+001F range can
        lead to rendering issues on some platforms.

        Acceptable control characters are defined as .null (U+0000) and
        CR (U+000D) for this check.
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/2430",
    title = "Does font file include unacceptable control character glyphs?"
)]
fn control_chars(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let codepoints = f.codepoints();
    let bad_characters = (0x01..0x1F)
        .filter(|&c| c != 0x0D)
        .filter(|c| codepoints.contains(c))
        .map(|c| format!("U+{:04X} ({})", c, f.glyph_name_for_unicode_synthesise(c)))
        .collect::<Vec<String>>();
    if bad_characters.is_empty() {
        Ok(Status::just_one_pass())
    } else {
        Ok(Status::just_one_fail(
            "unacceptable",
            &format!(
                "The following unacceptable control characters were identified:\n{}",
                bullet_list(context, &bad_characters)
            ),
        ))
    }
}
