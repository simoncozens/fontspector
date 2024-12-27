use std::collections::{HashMap, HashSet};

use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert, TestFont};
use read_fonts::{
    tables::stat::{AxisValue, AxisValueTableFlags},
    types::Fixed,
    ReadError, TableProvider,
};
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
        .stat()
        .map_err(|_| CheckError::skip("no-stat", "No STAT table"))?
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

fn segment_vf_collection(fonts: Vec<TestFont>) -> Vec<(Option<TestFont>, Option<TestFont>)> {
    let mut roman_italic = vec![];
    let (italics, mut non_italics): (Vec<_>, Vec<_>) = fonts
        .into_iter()
        .partition(|f| f.filename.to_str().unwrap_or_default().contains("-Italic["));
    for italic in italics.into_iter() {
        // Find a matching roman
        let suspected_roman = italic
            .filename
            .to_str()
            .unwrap_or_default()
            .replace("-Italic[", "[");
        if let Some(index) = non_italics
            .iter()
            .position(|f| f.filename.to_str().unwrap_or_default() == suspected_roman)
        {
            let roman = non_italics.swap_remove(index);
            roman_italic.push((Some(roman), Some(italic)));
        } else {
            roman_italic.push((None, Some(italic)));
        }
    }
    // Now add all the remaining non-italic fonts
    for roman in non_italics.into_iter() {
        roman_italic.push((Some(roman), None));
    }

    roman_italic
}

fn check_has_ital(t: &TestFont) -> Option<Status> {
    if let Ok(stat) = t.font().stat() {
        let has_ital = stat
            .design_axes()
            .ok()?
            .iter()
            .any(|axis| axis.axis_tag() == "ital");
        if !has_ital {
            Some(Status::fail(
                "missing-ital-axis",
                &format!(
                    "Font {} lacks an 'ital' axis in the STAT table.",
                    t.filename.to_string_lossy()
                ),
            ))
        } else {
            None
        }
    } else {
        Some(Status::fail(
            "no-stat",
            &format!("Font {} has no STAT table", t.filename.to_string_lossy()),
        ))
    }
}

// This is horrible because the structure of STAT table value records is horrible.
fn check_ital_is_binary_and_last(t: &TestFont, is_italic: bool) -> Result<Vec<Status>, ReadError> {
    let mut problems = vec![];
    if let Ok(stat) = t.font().stat() {
        let axes = stat.design_axes()?;
        if let Some(ital_pos) = axes.iter().position(|axis| axis.axis_tag() == "ital") {
            if ital_pos != axes.len() - 1 {
                problems.push(Status::warn(
                    "ital-axis-not-last",
                    &format!(
                        "Font {} has 'ital' axis in position {} of {}.",
                        t.filename.to_string_lossy(),
                        ital_pos + 1,
                        axes.len()
                    ),
                ));
            }

            let expected_value = if is_italic { 1.0 } else { 0.0 };
            let expected_flags = if is_italic {
                AxisValueTableFlags::empty()
            } else {
                AxisValueTableFlags::ELIDABLE_AXIS_VALUE_NAME
            };
            if let Some(Ok(subtable)) = stat.offset_to_axis_values() {
                for val in subtable.axis_values().iter().flatten() {
                    match &val {
                        AxisValue::Format1(v) => {
                            if v.axis_index() != ital_pos as u16 {
                                continue;
                            }
                            if v.value().to_f32() != expected_value {
                                problems.push(Status::warn(
                                    "wrong-ital-axis-value",
                                    &format!(
                                        "{} has STAT table 'ital' axis with wrong value. Expected: {}, got '{}'",
                                        t.filename.to_string_lossy(),
                                        expected_value,
                                        v.value()
                                    ),
                                ))
                            }
                            if val.flags() != expected_flags {
                                problems.push(Status::warn(
                                    "wrong-ital-axis-flag",
                                    &format!(
                                        "{} has STAT table 'ital' axis with wrong flags. Expected: {:?}, got '{:?}'",
                                        t.filename.to_string_lossy(),expected_flags,val.flags()
                                    ),
                                ))
                            }
                        }
                        AxisValue::Format2(v) => {
                            if v.axis_index() != ital_pos as u16 {
                                continue;
                            }
                            if v.nominal_value().to_f32() != expected_value {
                                problems.push(Status::warn(
                                    "wrong-ital-axis-value",
                                    &format!(
                                        "{} has STAT table 'ital' axis with wrong value. Expected: {}, got '{}'",
                                        t.filename.to_string_lossy(),
                                        expected_value,
                                        v.nominal_value()
                                    ),
                                ))
                            }
                            if val.flags() != expected_flags {
                                problems.push(Status::warn(
                                    "wrong-ital-axis-flag",
                                    &format!(
                                        "{} has STAT table 'ital' axis with wrong flags. Expected: {:?}, got '{:?}'",
                                        t.filename.to_string_lossy(),expected_flags,val.flags()
                                    ),
                                ))
                            }
                        }
                        AxisValue::Format3(v) => {
                            if v.axis_index() != ital_pos as u16 {
                                continue;
                            }
                            if v.value().to_f32() != expected_value {
                                problems.push(Status::warn(
                                    "wrong-ital-axis-value",
                                    &format!(
                                        "{} has STAT table 'ital' axis with wrong value. Expected: {}, got '{}'",
                                        t.filename.to_string_lossy(),
                                        expected_value,
                                        v.value()
                                    ),
                                ))
                            }
                            if val.flags() != expected_flags {
                                problems.push(Status::warn(
                                    "wrong-ital-axis-flag",
                                    &format!(
                                        "{} has STAT table 'ital' axis with wrong flags. Expected: {:?}, got '{:?}'",
                                        t.filename.to_string_lossy(),expected_flags,val.flags()
                                    ),
                                ))
                            }
                            // If we are Roman, check for the linked value
                            if !is_italic {
                                let linked_value = v.linked_value();
                                if linked_value.to_f32() != 1.0 {
                                    problems.push(Status::warn(
                                            "wrong-ital-axis-linkedvalue",
                                            &format!(
                                                "{} has STAT table 'ital' axis with wrong linked value. Expected: 1.0, got '{}'",
                                                t.filename.to_string_lossy(),
                                                linked_value
                                            ),
                                        ))
                                }
                            }
                        }
                        AxisValue::Format4(_) => {
                            // We don't handle this.
                        }
                    }
                }
            }
        }
    }
    Ok(problems)
}

#[check(
    id = "opentype/stat/ital_axis",
    rationale = "
        Check that related Upright and Italic VFs have an
        'ital' axis in the STAT table.
        
        Since the STAT table can be used to create new instances, it is
        important to ensure that such an 'ital' axis be the last one
        declared in the STAT table so that the eventual naming of new
        instances follows the subfamily traditional scheme (RIBBI / WWS)
        where "Italic" is always last.

        The 'ital' axis should also be strictly boolean, only accepting
        values of 0 (for Uprights) or 1 (for Italics). This usually works
        as a mechanism for selecting between two linked variable font files. 

        Also, the axis value name for uprights must be set as elidable.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/2934 and https://github.com/fonttools/fontbakery/issues/3668 and https://github.com/fonttools/fontbakery/issues/3669",
    implementation = "all",
    title = "Ensure VFs have 'ital' STAT axis."
)]
fn ital_axis(c: &TestableCollection, _context: &Context) -> CheckFnResult {
    let fonts = TTF.from_collection(c);
    let mut problems = vec![];

    for pair in segment_vf_collection(fonts).into_iter() {
        match pair {
            (Some(roman), Some(italic)) => {
                // These should definitely both have an ital axis
                problems.extend(check_has_ital(&roman));
                problems.extend(check_has_ital(&italic));
                problems.extend(check_ital_is_binary_and_last(&roman, false)?);
                problems.extend(check_ital_is_binary_and_last(&italic, true)?);
            }
            (None, Some(italic)) => {
                problems.push(Status::fail(
                    "missing-roman",
                    &format!(
                        "Italic font {} has no matching Roman font.",
                        italic.filename.to_string_lossy()
                    ),
                ));
            }
            (None, None) => {}
            (Some(roman), None) => {
                problems.extend(check_ital_is_binary_and_last(&roman, false)?);
            }
        }
    }
    return_result(problems)
}
