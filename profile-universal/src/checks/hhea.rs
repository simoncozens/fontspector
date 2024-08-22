use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;

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

pub const CHECK_MAXADVANCEWIDTH: Check = Check {
    id: "com.google.fonts/check/maxadvancewidth",
    title: "MaxAdvanceWidth is consistent with values in the Hmtx and Hhea tables?",
    rationale: "The 'hhea' table contains a field which specifies the maximum advance width. This value should be consistent with the maximum advance width of all glyphs specified in the 'hmtx' table.",
    proposal: "legacy:check/073",
    implementation: CheckImplementation::CheckOne(&maxadvancewidth),
    applies_to: "TTF",
    hotfix: None,
    fix_source: None,
    flags: CheckFlags::default(),
};

fn caret_slope(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let post_italic_angle = f.font().post()?.italic_angle().to_f32();
    let upem = f.font().head()?.units_per_em();
    let run = f.font().hhea()?.caret_slope_run();
    let rise = f.font().hhea()?.caret_slope_rise();
    if rise == 0 {
        return Ok(Status::just_one_fail(
            "zero-rise",
            "caretSlopeRise must not be zero. Set it to 1 for upright fonts.",
        ));
    }
    let hhea_angle = (-run as f32 / rise as f32).atan().to_degrees();
    let expected_run = (-post_italic_angle.to_radians().tan() * upem as f32).round() as i16;
    let expected_rise = if expected_run == 0 { 1 } else { upem };
    if (post_italic_angle - hhea_angle).abs() > 0.1 {
        return Ok(Status::just_one_warn(
            "mismatch",
            &format!(
                "hhea.caretSlopeRise and hhea.caretSlopeRun do not match with post.italicAngle.
                Got caretSlopeRise: {}, caretSlopeRun: {}, expected caretSlopeRise: {}, caretSlopeRun: {}",
                rise, run, expected_rise, expected_run
            ),
        ));
    }
    Ok(Status::just_one_pass())
}

pub const CHECK_CARET_SLOPE: Check = Check {
    id: "com.google.fonts/check/caret_slope",
    title: "Check hhea.caretSlopeRise and hhea.caretSlopeRun",
    rationale: r#"
        Checks whether hhea.caretSlopeRise and hhea.caretSlopeRun
        match with post.italicAngle.

        For Upright fonts, you can set hhea.caretSlopeRise to 1
        and hhea.caretSlopeRun to 0.

        For Italic fonts, you can set hhea.caretSlopeRise to head.unitsPerEm
        and calculate hhea.caretSlopeRun like this:
        round(math.tan(
          math.radians(-1 * font["post"].italicAngle)) * font["head"].unitsPerEm)

        This check allows for a 0.1° rounding difference between the Italic angle
        as calculated by the caret slope and post.italicAngle
    "#,
    proposal: "https://github.com/fonttools/fontbakery/issues/3670",
    implementation: CheckImplementation::CheckOne(&caret_slope),
    applies_to: "TTF",
    hotfix: None,
    fix_source: None,
    flags: CheckFlags::default(),
};
