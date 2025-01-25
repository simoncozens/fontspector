use fontspector_checkapi::{prelude::*, FileTypeConvert, TestFont};
use read_fonts::{
    tables::stat::{AxisValue, AxisValueTableFlags},
    ReadError, TableProvider,
};

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
    id = "opentype/STAT/ital_axis",
    rationale = "
        Check that related Upright and Italic VFs have an
        'ital' axis in the STAT table.
        
        Since the STAT table can be used to create new instances, it is
        important to ensure that such an 'ital' axis be the last one
        declared in the STAT table so that the eventual naming of new
        instances follows the subfamily traditional scheme (RIBBI / WWS)
        where \"Italic\" is always last.

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
