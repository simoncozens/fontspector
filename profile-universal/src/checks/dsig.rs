use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;
use skrifa::Tag;

#[check(
    id = "opentype/dsig",
    title = "The font should not need a DSIG table anymore.",
    rationale = "
        Microsoft Office 2013 and below products expect fonts to have a digital
        signature declared in a DSIG table in order to implement OpenType features.
        The EOL date for Microsoft Office 2013 products was 4/11/2023.

        This issue does not impact Microsoft Office 2016 and above products. It is now considered better to completely remove the table.

        But if you still want your font to support OpenType features on Office 2013,
        then you may find it handy to add a fake signature on a placeholder DSIG table
        by running one of the helper scripts provided at
        https://github.com/googlefonts/gftools

        Reference: https://github.com/fonttools/fontbakery/issues/1845
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3398"
)]
fn dsig(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let dsig_data = f.font().expect_data_for_tag(Tag::new(b"DSIG"));
    if dsig_data.is_ok() {
        Ok(Status::just_one_warn(
            "found-DSIG",
            "This font has a digital signature (DSIG table) which \
             is only required - even if only a placeholder \
             - on old programs like MS Office 2013 in order to \
             work properly.\n\
             The current recommendation is to completely \
             remove the DSIG table."
        ))
    } else {
        Ok(Status::just_one_pass())
    }
}
