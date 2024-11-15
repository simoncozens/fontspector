use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use skrifa::Tag;

const INSTRUCTIONS: [u8; 7] = [0xb8, 0x01, 0xff, 0x85, 0xb0, 0x04, 0x8d];

// This is gross, but saves bringing in memmem or twoway
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

#[check(
    id = "smart_dropout",
    rationale = "
        This setup is meant to ensure consistent rendering quality for fonts across
        all devices (with different rendering/hinting capabilities).

        Below is the snippet of instructions we expect to see in the fonts:
        
        ```
        B8 01 FF    PUSHW 0x01FF
        85          SCANCTRL (unconditinally turn on
                              dropout control mode)
        B0 04       PUSHB 0x04
        8D          SCANTYPE (enable smart dropout control)
        ```

        \"Smart dropout control\" means activating rules 1, 2 and 5:

        Rule 1: If a pixel's center falls within the glyph outline,
                that pixel is turned on.
        Rule 2: If a contour falls exactly on a pixel's center,
                that pixel is turned on.
        Rule 5: If a scan line between two adjacent pixel centers
                (either vertical or horizontal) is intersected
                by both an on-Transition contour and an off-Transition
                contour and neither of the pixels was already turned on
                by rules 1 and 2, turn on the pixel which is closer to
                the midpoint between the on-Transition contour and
                off-Transition contour. This is \"Smart\" dropout control.

        For more detailed info (such as other rules not enabled in this snippet),
        please refer to the TrueType Instruction Set documentation.

        Generally this occurs with unhinted fonts; if you are not using autohinting,
        use gftools-fix-nonhinting (or just gftools-fix-font) to fix this issue.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Ensure smart dropout control is enabled in \"prep\" table instructions."
)]
fn smart_dropout(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(
        !f.has_table(b"glyf"),
        "not-ttf",
        "Font does not have TrueType outlines"
    );

    skip!(
        f.has_table(b"TSI5"),
        "vtt-hinted",
        "Font is hinted with VTT"
    );
    // Except that other tests ensure this table is stripped out, oh well.

    Ok(
        if !f
            .font()
            .table_data(Tag::new(b"prep"))
            .map(|data| find_subsequence(data.as_bytes(), &INSTRUCTIONS))
            .unwrap_or(false)
        {
            Status::just_one_fail("lacks-smart-dropout", "The 'prep' table does not contain TrueType instructions enabling smart dropout control. To fix, export the font with autohinting enabled, or run ttfautohint on the font, or run the `gftools fix-nonhinting` script.")
        } else {
            Status::just_one_pass()
        },
    )
}
