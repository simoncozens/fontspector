use std::collections::{HashMap, HashSet};

use font_types::Fixed;
use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use read_fonts::{tables::stat::AxisValue, TableProvider};
use skrifa::MetadataProvider;

#[check(
    id = "opentype/varfont/stat_axis_record_for_each_axis",
    rationale = "
        According to the OpenType spec, there must be an Axis Record
        for every axis defined in the fvar table.

        https://docs.microsoft.com/en-us/typography/opentype/spec/stat#axis-records
    ",
    title = "All fvar axes have a correspondent Axis Record on STAT table?",
    proposal = "https://github.com/fonttools/fontbakery/pull/3017"
)]
fn stat_axis_record(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.is_variable_font(), "not-variable", "Not a variable font");
    let fvar_axis_tags: HashSet<_> = f
        .font()
        .axes()
        .iter()
        .map(|axis| axis.tag().to_string())
        .collect();
    let stat_axis_tags: HashSet<_> = f
        .font()
        .stat()?
        .design_axes()?
        .iter()
        .map(|axis_record| axis_record.axis_tag().to_string())
        .collect();
    let missing_axes: Vec<&str> = fvar_axis_tags
        .difference(&stat_axis_tags)
        .map(|x| x.as_ref())
        .collect();
    Ok(if missing_axes.is_empty() {
        Status::just_one_pass()
    } else {
        Status::just_one_fail(
            "missing-axis-records",
            &format!(
                "STAT table is missing Axis Records for the following axes:\n\n{}",
                bullet_list(context, &missing_axes)
            ),
        )
    })
}

#[check(
    id = "opentype/stat_has_axis_value_tables",
    rationale = "
            According to the OpenType spec, in a variable font, it is strongly recommended
            that axis value tables be included for every element of typographic subfamily
            names for all of the named instances defined in the 'fvar' table.
    
            Axis value tables are particularly important for variable fonts, but can also
            be used in non-variable fonts. When used in non-variable fonts, axis value
            tables for particular values should be implemented consistently across fonts
            in the family.
    
            If present, Format 4 Axis Value tables are checked to ensure they have more than
            one AxisValueRecord (a strong recommendation from the OpenType spec).
    
            https://docs.microsoft.com/en-us/typography/opentype/spec/stat#axis-value-tables
        ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3090",
    title = "STAT table has Axis Value tables"
)]
fn stat_has_axis_value_tables(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.has_table(b"STAT"), "no-STAT", "Font lacks a STAT table");
    let stat = f.font().stat()?;
    let mut messages = vec![];
    if stat.axis_value_count() == 0 {
        return Ok(Status::just_one_fail(
            "no-axis-value-tables",
            "STAT table has no Axis Value tables.",
        ));
    }
    let mut stat_axis_values = HashMap::new();
    for (index, axis) in stat.design_axes()?.iter().enumerate() {
        let tag = axis.axis_tag().to_string();
        let mut axis_values = HashSet::new();
        if let Some(Ok(subtable)) = stat.offset_to_axis_values() {
            for axis_value in subtable.axis_values().iter().flatten() {
                match axis_value {
                    AxisValue::Format1(av) => {
                        if index == av.axis_index() as usize {
                            axis_values.insert(av.value());
                        }
                    }
                    AxisValue::Format2(av) => {
                        if index == av.axis_index() as usize {
                            axis_values.insert(av.nominal_value());
                        }
                    }
                    AxisValue::Format3(av) => {
                        if index == av.axis_index() as usize {
                            axis_values.insert(av.value());
                        }
                    }
                    AxisValue::Format4(av) => {
                        if av.axis_values().len() < 2 {
                            messages.push(Status::fail(
                                "format-4-axis-count",
                                &format!(
                                    "Axis Value table for axis '{}' has only one AxisValueRecord.",
                                    tag
                                ),
                            ));
                        }
                    }
                }
            }
        }
        stat_axis_values.insert(tag, axis_values);
    }

    for (_, coord) in f.named_instances() {
        for (axis, value) in coord.iter() {
            if let Some(axis_values) = stat_axis_values.get(axis) {
                if !axis_values.contains(&Fixed::from_f64(*value as f64)) {
                    messages.push(Status::fail(
                        "missing-axis-value-table",
                        &format!(
                            "Axis Value table for axis '{}' is missing a value '{}'.",
                            axis, value
                        ),
                    ));
                }
            }
        }
    }

    return_result(messages)
}

#[check(
    id = "opentype/weight_class_fvar",
    rationale = "According to Microsoft's OT Spec the OS/2 usWeightClass should match the fvar default value.",
    proposal = "https://github.com/googlefonts/gftools/issues/477",
    title = "Checking if OS/2 usWeightClass matches fvar."
)]
fn weight_class_fvar(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.is_variable_font(), "not-variable", "Not a variable font");
    let fvar_value = f
        .axis_ranges()
        .find(|(tag, _, _, _)| tag == "wght")
        .map(|(_, _, default, _)| default)
        .ok_or(CheckError::skip("no-wght", "No 'wght' axis"))?;
    let os2_value = f
        .font()
        .os2()
        .map_err(|_| CheckError::skip("no-os2", "No OS/2 table"))?
        .us_weight_class();
    if os2_value != fvar_value as u16 {
        return Ok(Status::just_one_fail(
            "bad-weight-class",
            &format!(
                "OS/2 usWeightClass is {}, but fvar default is {}",
                os2_value, fvar_value
            ),
        ));
    }

    Ok(Status::just_one_pass())
}
