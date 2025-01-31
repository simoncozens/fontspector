use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use hashbrown::{HashMap, HashSet};
use skrifa::MetadataProvider;

use crate::{
    checks::googlefonts::metadata::family_proto,
    network_conditions::{is_listed_on_google_fonts, remote_styles},
};

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
    let remote_styles = remote_styles(family, context).map_err(CheckError::Error)?;
    let mut missing_axes = HashSet::new();

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
        let our_subfamily_name = f.best_subfamilyname();
        let remote_style = remote_styles
            .iter()
            .flat_map(|s| TTF.from_testable(s))
            .find(|remote_f| remote_f.best_subfamilyname() == our_subfamily_name)
            .ok_or(CheckError::Error(format!(
                "No matching remote style for {}",
                t.basename().unwrap_or("Regular".to_string())
            )))?;
        let remote_axes = remote_style
            .font()
            .axes()
            .iter()
            .map(|a| (a.tag(), (a.min_value(), a.max_value())));
        let font_axes: HashMap<_, _> = f
            .font()
            .axes()
            .iter()
            .map(|a| (a.tag(), (a.min_value(), a.max_value())))
            .collect();
        for (axis, range) in remote_axes {
            if let Some(remote_range) = font_axes.get(&axis) {
                let (axis_min, axis_max) = remote_range;
                let (remote_axis_min, remote_axis_max) = range;
                if *axis_min > remote_axis_min {
                    problems.push(Status::fail(
                        "axis-min-out-of-range",
                        &format!(
                            "Axis '{}' min value is out of range. Expected '{}', got '{}'.",
                            axis, remote_axis_min, axis_min
                        ),
                    ));
                }
                if *axis_max < remote_axis_max {
                    problems.push(Status::fail(
                        "axis-max-out-of-range",
                        &format!(
                            "Axis '{}' max value is out of range. Expected '{}', got '{}'.",
                            axis, remote_axis_max, axis_max
                        ),
                    ));
                }
            } else {
                missing_axes.insert(axis);
                continue;
            }
        }
    }
    if !missing_axes.is_empty() {
        problems.push(Status::fail(
            "missing-axes",
            &format!(
                "Missing axes: {}",
                missing_axes
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        ));
    }
    return_result(problems)
}
