use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;

const FSTYPE_RESTRICTIONS: [(u16, &str); 5] = [
    (0x0002,
        "* The font must not be modified, embedded or exchanged in any manner without first obtaining permission of the legal owner."
    ),
    (0x0004,
        "* The font may be embedded, and temporarily loaded on the remote system, but documents that use it must not be editable."
    ),
    (0x0008,
        "* The font may be embedded but must only be installed temporarily on other systems."
    ),
    (0x0100, "* The font may not be subsetted prior to embedding."),
    (0x0200,
        "* Only bitmaps contained in the font may be embedded. No outline data may be embedded."
    ),
];

#[check(
    id = "googlefonts/fstype",
    rationale = "
        
        The fsType in the OS/2 table is a legacy DRM-related field. Fonts in the
        Google Fonts collection must have it set to zero (also known as
        \"Installable Embedding\"). This setting indicates that the fonts can be
        embedded in documents and permanently installed by applications on
        remote systems.

        More detailed info is available at:
        https://docs.microsoft.com/en-us/typography/opentype/spec/os2#fstype
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Checking OS/2 fsType does not impose restrictions."
)]
fn fstype(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let fstype_value = f.font().os2()?.fs_type();
    if fstype_value == 0 {
        return Ok(Status::just_one_pass());
    }
    let mut restrictions = FSTYPE_RESTRICTIONS
        .iter()
        .filter(|(bit_mask, _)| fstype_value & bit_mask != 0)
        .map(|(_, restriction)| restriction.to_string())
        .collect::<Vec<String>>();
    if fstype_value & 0b1111110011110001 != 0 {
        restrictions
            .push("* There are reserved bits set, which indicates an invalid setting.".to_string());
    }
    Ok(Status::just_one_fail(
        "drm",
        &format!(
            "In this font fsType is set to {} meaning that:\n{}\n\nNo such DRM restrictions can be enabled on the Google Fonts collection, so the fsType field must be set to zero (Installable Embedding) instead.",
            fstype_value,
            restrictions.join("\n")
        ),
    ))
}
