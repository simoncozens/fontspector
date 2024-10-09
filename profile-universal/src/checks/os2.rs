use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert, TestFont};
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

#[check(
    id = "opentype/family/panose_familytype",
    proposal = "legacy:check/010",
    title = "Fonts have consistent PANOSE family type?",
    rationale = "
    The [PANOSE value](https://monotype.github.io/panose/) in the OS/2 table is a
    way of classifying a font based on its visual appearance and characteristics.

    The first field in the PANOSE classification is the family type: 2 means Latin
    Text, 3 means Latin Script, 4 means Latin Decorative, 5 means Latin Symbol.
    This check ensures that within a family, all fonts have the same family type.
    ",
    implementation = "all"
)]
fn panose_familytype(c: &TestableCollection, _context: &Context) -> CheckFnResult {
    let fonts = TTF.from_collection(c);
    let mut problems = vec![];
    let (ok, missing): (Vec<&TestFont>, Vec<&TestFont>) =
        fonts.iter().partition(|f| f.font().os2().is_ok());
    for font in missing {
        problems.push(Status::error(&format!(
            "Font {} is missing an OS/2 table",
            font.filename.to_string_lossy()
        )));
    }
    if !problems.is_empty() {
        return return_result(problems);
    }
    let panose_values = ok
        .iter()
        .map(|f| {
            #[allow(clippy::unwrap_used)]
            let panose_first = f.font().os2().unwrap().panose_10()[0];
            let panose_name = match panose_first {
                2 => "Latin Text".to_string(),
                3 => "Latin Script".to_string(),
                4 => "Latin Decorative".to_string(),
                5 => "Latin Symbol".to_string(),
                _ => format!("Unknown ({})", panose_first),
            };

            #[allow(clippy::unwrap_used)]
            (
                panose_first,
                panose_name,
                f.filename.file_name().unwrap().to_string_lossy(),
            )
        })
        .collect::<Vec<_>>();
    assert_all_the_same(
        _context,
        &panose_values,
        "inconsistency",
        "PANOSE family type is not the same across this family. In order to fix this, please make sure that the panose.bFamilyType value is the same in the OS/2 table of all of this family's font files.",
    )
}

#[check(
    id = "opentype/vendor_id",
    rationale = "
        When a font project's Vendor ID is specified explicitly on FontBakery's
        configuration file, all binaries must have a matching vendor identifier
        value in the OS/2 table.
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/3941",
    title = "Check OS/2 achVendID against configuration"
)]
fn check_vendor_id(f: &Testable, context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    let vendor_id = context
        .check_metadata
        .get("vendor_id")
        .ok_or(CheckError::skip(
            "no-vendor-id",
            "Add the `vendor_id` key to a `fontspector.yaml` file on your font project directory to enable this check.\nYou'll also need to use the `--configuration` flag when invoking fontspector",
        ))?;
    let os2_vendor_id = font.font().os2()?.ach_vend_id().to_string();
    if os2_vendor_id.as_str() == vendor_id {
        Ok(Status::just_one_pass())
    } else {
        Ok(Status::just_one_fail(
            "bad-vendor-id",
            &format!(
                "OS/2 achVendID value '{}' does not match configuration value '{}'",
                os2_vendor_id, vendor_id
            ),
        ))
    }
}
