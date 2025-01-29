use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;

const RECOMMENDED: [u16; 11] = [16, 32, 64, 128, 256, 500, 512, 1000, 1024, 2000, 2048];

#[check(
    id = "googlefonts/unitsperem",
    rationale = "
        
        Even though the OpenType spec allows unitsPerEm to be any value between 16
        and 16384, the Google Fonts project aims at a narrower set of reasonable values.

        Values above 4000 would likely result in unreasonable filesize increases.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Stricter unitsPerEm criteria for Google Fonts."
)]
fn unitsperem(t: &Testable, _context: &Context) -> CheckFnResult {
    let upm = testfont!(t).font().head()?.units_per_em();
    Ok(if upm > 4000 {
        Status::just_one_fail("large-value", &format!(
                "Font em size (unitsPerEm) is {} which may be too large (causing filesize bloat), unless you are sure that the detail level in this font requires that much precision.",
                upm
            ))
    } else if upm < 16 {
        Status::just_one_fail("bad-value", &format!(
                "Font em size (unitsPerEm) is {}. If possible, please consider using 1000. Good values for unitsPerEm, though, are typically these: {:?}.",
                upm, RECOMMENDED
            ))
    } else {
        Status::just_one_pass()
    })
}
