use fontspector_checkapi::{prelude::*, skip, FileTypeConvert};
use google_fonts_axisregistry::AxisRegistry;
use hashbrown::HashSet;
use skrifa::MetadataProvider;

use crate::checks::googlefonts::metadata::family_proto;

#[check(
    id = "googlefonts/metadata/axes",
    rationale = "
        
        Each axis range in a METADATA.pb file must be registered, and within the bounds
        of the axis definition in the Google Fonts Axis Registry, available at
        https://github.com/google/fonts/tree/main/axisregistry
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3010 and https://github.com/fonttools/fontbakery/issues/3022",
    title = "Validate METADATA.pb axes values.",
    implementation = "all"
)]
fn axes(c: &TestableCollection, context: &Context) -> CheckFnResult {
    let mut problems = vec![];
    let mdpb = c
        .get_file("METADATA.pb")
        .ok_or_else(|| CheckError::skip("no-mdpb", "No METADATA.pb file found"))?;
    let msg = family_proto(mdpb)?;
    // Skip if no variable fonts
    let fonts = msg
        .fonts
        .iter()
        .flat_map(|f| f.filename.as_ref())
        .flat_map(|f| c.get_file(f))
        .flat_map(|f| TTF.from_testable(f))
        .collect::<Vec<_>>();
    if !fonts.iter().any(|f| f.is_variable_font()) {
        skip!("no-variable", "No variable fonts found in the family");
    }

    let axisregistry = AxisRegistry::new();
    for axis in msg.axes.iter() {
        if let Some(registry_tag) = axisregistry.get(axis.tag()) {
            // Check bounds are correct, googlefonts/metadata/axisregistry_bounds
            if axis.min_value() < registry_tag.min_value()
                || axis.max_value() > registry_tag.max_value()
            {
                problems.push(Status::fail(
                    "bad-axis-range",
                    &format!(
                        "The range in the font variation axis '{}' ({}) min:{} max:{} does not comply with the expected maximum range, as defined on Google Fonts Axis Registry (min:{} max:{}).",
                        axis.tag(),
                        registry_tag.display_name(),
                        axis.min_value(),
                        axis.max_value(),
                        registry_tag.min_value(),
                        registry_tag.max_value()
                    ),
                ));
            }
        } else {
            // googlefonts/metadata/axisregistry_valid_tags, as was.
            problems.push(Status::fail(
                "bad-axis-tag",
                &format!(
                    "The font variation axis '{}' is not yet registered in the Google Fonts Axis Registry",
                    axis.tag()
                ),
            ));
        }
    }

    // googlefonts/metadata/consistent_axis_enumeration

    // Let's get the set of axes in the fonts
    let font_axes = fonts
        .iter()
        .flat_map(|f| f.font().axes().iter())
        .map(|ax| ax.tag().to_string())
        .collect::<HashSet<_>>();
    let md_axes = msg
        .axes
        .iter()
        .map(|ax| ax.tag().to_string())
        .collect::<HashSet<_>>();
    let missing = font_axes.difference(&md_axes).collect::<Vec<_>>();
    let extra = md_axes.difference(&font_axes).collect::<Vec<_>>();

    if !missing.is_empty() {
        problems.push(Status::fail(
            "missing-axes",
            &format!(
                "The font variation axes {} are present in the font's fvar table but are not declared on the METADATA.pb file.",
                bullet_list(context, &missing)
            ),
        ));
    }
    if !extra.is_empty() {
        problems.push(Status::fail(
            "extra-axes",
            &format!(
                "The METADATA.pb file lists font variation axes that are not supported by this family: {}",
                bullet_list(context, &extra)
            ),
        ));
    }

    return_result(problems)
}
