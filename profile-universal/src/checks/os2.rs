use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::{
    tables::{head::MacStyle, os2::SelectionFlags},
    TableProvider,
};

#[check(
    id = "opentype/fsselection",
    proposal = "legacy:check/129",
    title = "Checking OS/2 fsSelection value.",
    rationale = "
    The OS/2.fsSelection field is a bit field used to specify the stylistic
    qualities of the font - in particular, it specifies to some operating
    systems whether the font is italic (bit 0), bold (bit 5) or regular
    (bit 6).

    This check verifies that the fsSelection field is set correctly for the
    font style. For a family of static fonts created in GlyphsApp, this is
    set by using the style linking checkboxes in the exports settings.

    Additionally, the bold and italic bits in OS/2.fsSelection must match
    the bold and italic bits in head.macStyle per the OpenType spec.
    "
)]
fn fsselection(f: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    let fs_flags = font.font().os2()?.fs_selection();
    let style = font
        .style()
        .ok_or(CheckError::skip("no-style", "No style detected"))?;
    let bold_expected = style == "Bold" || style == "BoldItalic";
    let italic_expected = style.contains("Italic");
    let regular_expected = !bold_expected && !italic_expected;
    let mut problems = vec![];
    let bold_seen = fs_flags.contains(SelectionFlags::BOLD);
    let italic_seen = fs_flags.contains(SelectionFlags::ITALIC);
    let regular_seen = fs_flags.contains(SelectionFlags::REGULAR);
    for (flag, expected, label) in &[
        (bold_seen, bold_expected, "Bold"),
        (italic_seen, italic_expected, "Italic"),
        (regular_seen, regular_expected, "Regular"),
    ] {
        if flag != expected {
            problems.push(Status::warn(
                "mismatch",
                &format!(
                    "fsSelection {} flag {} does not match font style {}",
                    label, flag, style
                ),
            ));
        }
    }

    let mac_style_bits = font.font().head()?.mac_style();
    let mac_bold = mac_style_bits.contains(MacStyle::BOLD);
    let mac_italic = mac_style_bits.contains(MacStyle::ITALIC);
    for (flag, expected, label) in &[
        (bold_seen, mac_bold, "Bold"),
        (italic_seen, mac_italic, "Italic"),
    ] {
        if flag != expected {
            problems.push(Status::warn(
                "macstyle-mismatch",
                &format!(
                    "fsSelection {} flag {} does not match macStyle {} flag",
                    label, flag, expected
                ),
            ));
        }
    }
    return_result(problems)
}
