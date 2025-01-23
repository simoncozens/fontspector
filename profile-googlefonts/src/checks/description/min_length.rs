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
fn min_length(desc: &Testable, _context: &Context) -> CheckFnResult {
    Ok(if desc.contents.len() <= 200 {
        Status::just_one_fail(
            "too-short",
            "DESCRIPTION.en_us.html must have size larger than 200 bytes.",
        )
    } else {
        Status::just_one_pass()
    })
}
