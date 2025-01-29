use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert, TestFont};
use google_fonts_axisregistry::AxisRegistry;
use hashbrown::HashMap;
use read_fonts::{tables::stat::AxisValue, TableProvider};
use skrifa::{string::StringId, Tag};

fn normalize_name(name: &str) -> String {
    name.split_whitespace().collect()
}

fn process_axis(
    problems: &mut Vec<Status>,
    axis_registry: &AxisRegistry,
    axis_index: usize,
    axis_value: f32,
    value_name_id: StringId,
    f: &TestFont,
    axes: &[Tag],
) -> Result<(), CheckError> {
    let axis = axes.get(axis_index).ok_or(CheckError::Error(
        "Axis not found in STAT table".to_string(),
    ))?;
    if *axis == Tag::new(b"MORF") {
        return Ok(());
    }
    if let Some(entry) = axis_registry.get(&axis.to_string()) {
        let fallbacks = &entry.fallback;
        let name_entry =
            f.get_name_entry_strings(value_name_id)
                .next()
                .ok_or(CheckError::Error(
                    "Name reference in STAT table not found in name table".to_string(),
                ))?;
        // Here "name_entry" has the user-friendly name of the current AxisValue
        // We want to ensure that this string shows up as a "fallback" name
        // on the GF Axis Registry for this specific variation axis tag.

        let name = normalize_name(&name_entry);
        let expected_names_values = fallbacks
            .iter()
            .map(|f| (normalize_name(f.name()), f.value()))
            .collect::<HashMap<_, _>>();
        if !expected_names_values.contains_key(&name) {
            let expected_names = expected_names_values
                .keys()
                .map(|k| k.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            problems.push(Status::fail(
            "invalid-name",
            &format!(
                "On the font variation axis '{}', the name '{}' is not among the expected ones ({}) according to the Google Fonts Axis Registry.",
                axis,
                name_entry,
                expected_names
            ),
        ));
        } else if axis_value != expected_names_values[&name] {
            problems.push(Status::fail(
                "bad-coordinate",
                &format!(
                    "Axis Value for '{}':'{}' is expected to be '{}' but this font has '{}'.",
                    axis, name_entry, expected_names_values[&name], axis_value
                ),
            ));
        }
    }
    Ok(())
}

#[check(
    id = "googlefonts/STAT/axisregistry",
    rationale = "
        
        Check that particle names and values on STAT table match the fallback names
        in each axis entry at the Google Fonts Axis Registry, available at
        https://github.com/google/fonts/tree/main/axisregistry
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3022",
    title = "
    Validate STAT particle names and values match the fallback names in GFAxisRegistry.
    "
)]
fn axisregistry(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    let mut format4_entries = false;
    skip!(
        !f.is_variable_font(),
        "not-variable",
        "Font is not a variable font."
    );
    let axis_registry = AxisRegistry::new();
    if let Ok(stat) = f.font().stat() {
        let axes = stat
            .design_axes()?
            .iter()
            .map(|axis| axis.axis_tag())
            .collect::<Vec<_>>();
        if let Some(Ok(subtable)) = stat.offset_to_axis_values() {
            for axis_value in subtable.axis_values().iter().flatten() {
                match axis_value {
                    AxisValue::Format4(table_ref) => {
                        let mut coords = vec![];
                        for record in table_ref.axis_values() {
                            let axis =
                                axes.get(record.axis_index() as usize)
                                    .ok_or(CheckError::Error(
                                        "Axis not found in STAT table".to_string(),
                                    ))?;
                            coords.push(format!("{}:{}", axis, record.value()));
                        }
                        let coords = coords.join(", ");
                        let name = f
                            .get_name_entry_strings(table_ref.value_name_id())
                            .next()
                            .ok_or(CheckError::Error(
                                "Name reference in STAT table not found in name table".to_string(),
                            ))?;
                        format4_entries = true;
                        problems.push(Status::info(
                            "format-4",
                            &format!("'{}' at ({})", name, coords),
                        ));
                    }
                    AxisValue::Format1(table_ref) => {
                        process_axis(
                            &mut problems,
                            &axis_registry,
                            table_ref.axis_index() as usize,
                            table_ref.value().to_f32(),
                            table_ref.value_name_id(),
                            &f,
                            &axes,
                        )?;
                    }
                    AxisValue::Format2(table_ref) => {
                        process_axis(
                            &mut problems,
                            &axis_registry,
                            table_ref.axis_index() as usize,
                            table_ref.nominal_value().to_f32(),
                            table_ref.value_name_id(),
                            &f,
                            &axes,
                        )?;
                    }
                    AxisValue::Format3(table_ref) => {
                        process_axis(
                            &mut problems,
                            &axis_registry,
                            table_ref.axis_index() as usize,
                            table_ref.value().to_f32(),
                            table_ref.value_name_id(),
                            &f,
                            &axes,
                        )?;
                    }
                }
            }
            if format4_entries {
                problems.push(Status::info(
                    "format-4",
                    "The GF Axis Registry does not currently contain fallback names for the combination of values for more than a single axis, which is what these 'format 4' entries are designed to describe, so this check will ignore them for now.",
                ));
            }
            return_result(problems)
        } else {
            return Ok(Status::just_one_fail(
                "missing-axis-values",
                "STAT table is missing Axis Value Records",
            ));
        }
    } else {
        return Ok(Status::just_one_fail(
            "missing-stat",
            "Font is missing STAT table.",
        ));
    }
}
