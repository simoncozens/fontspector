use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};

use crate::{metadata::family_proto, network_conditions::is_listed_on_google_fonts};

#[check(
    id = "googlefonts/axes_match",
    rationale = "
        An updated font family must include the same axes found in the Google
        Fonts version, with the same axis ranges.
    ",
    proposal = "None",
    title = "Check if the axes match between the font and the Google Fonts version.",
    implementation = "all"
)]
fn axes_match(c: &TestableCollection, context: &Context) -> CheckFnResult {
    let mdpb = c
        .get_file("METADATA.pb")
        .ok_or_else(|| CheckError::skip("no-mdpb", "No METADATA.pb file found"))?;
    let msg = family_proto(mdpb)?;
    let fonts = msg
        .fonts
        .iter()
        .flat_map(|f| f.filename.as_ref())
        .flat_map(|f| c.get_file(f))
        .collect::<Vec<&Testable>>();
    let name = msg.name().to_string();
    let family = msg.display_name.as_ref().unwrap_or(&name);
    let mut problems: Vec<Status> = vec![];
    for t in fonts.iter() {
        let f = testfont!(t);
        skip!(
            context.skip_network,
            "network-check",
            "Skipping network check"
        );
        skip!(!f.is_variable_font(), "not-variable", "Not a variable font");
        skip!(
            !is_listed_on_google_fonts(family, context).map_err(CheckError::Error)?,
            "not-listed",
            "Not listed on Google Fonts"
        );
    }
    let mut problems = vec![];
    //     remote_axes = {
    //         a.axisTag: (a.minValue, a.maxValue) for a in remote_style["fvar"].axes
    //     }
    //     font_axes = {a.axisTag: (a.minValue, a.maxValue) for a in ttFont["fvar"].axes}
    //
    //     missing_axes = []
    //     for axis, remote_axis_range in remote_axes.items():
    //         if axis not in font_axes:
    //             missing_axes.append(axis)
    //             continue
    //         axis_range = font_axes[axis]
    //         axis_min, axis_max = axis_range
    //         remote_axis_min, remote_axis_max = remote_axis_range
    //         if axis_min > remote_axis_min:
    //             yield FAIL, Message(
    //                 "axis-min-out-of-range",
    //                 f"Axis '{axis}' min value is out of range."
    //                 f" Expected '{remote_axis_min}', got '{axis_min}'.",
    //             )
    //         if axis_max < remote_axis_max:
    //             yield FAIL, Message(
    //                 "axis-max-out-of-range",
    //                 f"Axis {axis} max value is out of range."
    //                 f" Expected {remote_axis_max}, got {axis_max}.",
    //             )
    //
    //     if missing_axes:
    //         yield FAIL, Message(
    //             "missing-axes",
    //             f"Missing axes: {', '.join(missing_axes)}",
    //         )
    //     else:
    //         yield PASS, "Axes match Google Fonts version."
    //
    // unimplemented!();
    return_result(problems)
}
