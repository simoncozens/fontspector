use std::ascii::escape_default;

use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::types::{CFF_SFNT_VERSION, TT_SFNT_VERSION};

fn escape_bytes<B: AsRef<[u8]>>(buf: B) -> String {
    String::from_utf8(
        buf.as_ref()
            .iter()
            .copied()
            .flat_map(escape_default)
            .collect(),
    )
    .unwrap_or("<invalid utf8>".to_string())
}

#[check(
    id = "sfnt_version",
    rationale = "
        OpenType fonts that contain TrueType outlines should use the value of 0x00010000
        for the sfntVersion. OpenType fonts containing CFF data (version 1 or 2) should
        use 0x4F54544F ('OTTO', when re-interpreted as a Tag) for sfntVersion.

        Fonts with the wrong sfntVersion value are rejected by FreeType.

        https://docs.microsoft.com/en-us/typography/opentype/spec/otff#table-directory
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3388",
    title = "Font has the proper sfntVersion value?"
)]
fn sfnt_version(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let font_sfnt_version = f.font().table_directory.sfnt_version();
    if f.has_table(b"glyf") && font_sfnt_version != TT_SFNT_VERSION {
        return Ok(Status::just_one_fail(
            "wrong-sfnt-version-ttf",
            &format!(
                "Font with TrueType outlines has incorrect sfntVersion value: '{:}'",
                escape_bytes(font_sfnt_version.to_be_bytes())
            ),
        ));
    }
    if (f.has_table(b"CFF ") || f.has_table(b"CFF2")) && font_sfnt_version != CFF_SFNT_VERSION {
        return Ok(Status::just_one_fail(
            "wrong-sfnt-version-cff",
            &format!(
                "Font with CFF data has incorrect sfntVersion value: '{:}'",
                escape_bytes(font_sfnt_version.to_be_bytes())
            ),
        ));
    }
    Ok(Status::just_one_pass())
}
