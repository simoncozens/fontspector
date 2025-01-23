use fontspector_checkapi::prelude::*;

#[check(
    id = "googlefonts/description/eof_linebreak",
    title = "DESCRIPTION.en_us.html should end in a linebreak.",
    rationale = "
        Some older text-handling tools sometimes misbehave if the last line of data
        in a text file is not terminated with a newline character (also known as '\\n').

        We know that this is a very small detail, but for the sake of keeping all
        DESCRIPTION.en_us.html files uniformly formatted throughout the GFonts
        collection, we chose to adopt the practice of placing this final linebreak
        character on them.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/2879",
    applies_to = "DESC"
)]
fn eof_linebreak(desc: &Testable, _context: &Context) -> CheckFnResult {
    Ok(if !desc.contents.ends_with(b"\n") {
        Status::just_one_warn(
            "missing-eof-linebreak",
            "The last characther on DESCRIPTION.en_us.html \
             is not a line-break. Please add it.",
        )
    } else {
        Status::just_one_pass()
    })
}
