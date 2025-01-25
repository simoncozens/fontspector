use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;

#[check(
    id = "opentype/maxadvancewidth",
    title = "MaxAdvanceWidth is consistent with values in the Hmtx and Hhea tables?",
    rationale = "
        The 'hhea' table contains a field which specifies the maximum advance width.
        This value should be consistent with the maximum advance width of all glyphs
        specified in the 'hmtx' table.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",  // legacy check
)]
fn maxadvancewidth(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let hhea_advance_width_max = f.font().hhea()?.advance_width_max().to_u16();
    let hmtx_advance_width_max = f
        .font()
        .hmtx()?
        .h_metrics()
        .iter()
        .map(|m| m.advance.get())
        .max()
        .unwrap_or_default();
    Ok(if hmtx_advance_width_max != hhea_advance_width_max {
        Status::just_one_fail(
            "mismatch",
            &format!(
                "AdvanceWidthMax mismatch: expected {} from hmtx; got {} for hhea",
                hmtx_advance_width_max, hhea_advance_width_max
            ),
        )
    } else {
        Status::just_one_pass()
    })
}
