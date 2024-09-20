use fontspector_checkapi::prelude::*;

#[check(
    id = "googlefonts/description/min_length",
    title = "DESCRIPTION.en_us.html must have more than 200 bytes.",
    rationale = "
        The DESCRIPTION.en_us.html file is intended to provide a brief overview of
        the font family. It should be long enough to be useful to users, but not so
        long that it becomes overwhelming.

        We chose 200 bytes as a minimum length because it suggests that someone has
        taken the time to write \"something sensible\" about the font.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",  // legacy check
    applies_to = "DESC"
)]
fn description_min_length(desc: &Testable, _context: &Context) -> CheckFnResult {
    Ok(if desc.contents.len() <= 200 {
        Status::just_one_fail("too-short",
            "DESCRIPTION.en_us.html must have size larger than 200 bytes.",
        )
    } else {
        Status::just_one_pass()
    })
}

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
    proposal="https://github.com/fonttools/fontbakery/issues/2879",
    applies_to = "DESC"
)]
fn description_eof_linebreak(desc: &Testable, _context: &Context) -> CheckFnResult {
    Ok(if desc.contents.ends_with(b"\n") {
        Status::just_one_warn("missing-eof-linebreak",
            "The last characther on DESCRIPTION.en_us.html \
             is not a line-break. Please add it.",
        )
    } else {
        Status::just_one_pass()
    })
}
