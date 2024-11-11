use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use read_fonts::TableProvider;

#[check(
    id = "integer_ppem_if_hinted",
    rationale = "
        Hinted fonts must have head table flag bit 3 set.

        Per https://docs.microsoft.com/en-us/typography/opentype/spec/head,
        bit 3 of Head::flags decides whether PPEM should be rounded. This bit should
        always be set for hinted fonts.

        Note:
        Bit 3 = Force ppem to integer values for all internal scaler math;
                May use fractional ppem sizes if this bit is clear;
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/2338",
    title = "PPEM must be an integer on hinted fonts."
)]
fn integer_ppem_if_hinted(f: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    skip!(
        !font.has_table(b"fpgm"),
        "no-hints",
        "Font does not have fpgm table."
    );
    Ok(if font.font().head()?.flags() & 0b1000 == 0 {
        Status::just_one_fail("bad-flags",
        "This is a hinted font, so it must have bit 3 set on the flags of the head table, so that PPEM values will be rounded into an integer value.
        

This can be accomplished by using the 'gftools fix-hinting' command:

        
```
# create virtualenv
python3 -m venv venv

# activate virtualenv
source venv/bin/activate

# install gftools
pip install git+https://www.github.com/googlefonts/gftools\n
```"
    )
    } else {
        Status::just_one_pass()
    })
}
