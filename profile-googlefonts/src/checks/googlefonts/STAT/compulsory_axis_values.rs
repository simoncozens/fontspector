use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use hashbrown::HashMap;
use itertools::Itertools;
use markdown_table::{Heading, MarkdownTable};
use read_fonts::{tables::stat::AxisValue, TableProvider};
use skrifa::{FontRef, MetadataProvider};

use crate::utils::build_expected_font;

const AXES_TO_CHECK: [&str; 10] = [
    "CASL", "CRSV", "FILL", "FLAR", "MONO", "SOFT", "VOLM", "wdth", "wght", "WONK",
];

#[derive(Debug, PartialEq)]
struct SimpleAxisValue {
    flags: u16,
    value: f32,
    linked_value: Option<f32>,
}

fn stat_axis_values(f: &FontRef) -> Result<HashMap<(String, String), SimpleAxisValue>, CheckError> {
    let stat = f.stat()?;
    let mut res = HashMap::new();
    let axes = stat
        .design_axes()?
        .iter()
        .map(|a| a.axis_tag())
        .collect::<Vec<_>>();
    if let Some(Ok(subtable)) = stat.offset_to_axis_values() {
        for axis_value in subtable.axis_values().iter().flatten() {
            let nameid = axis_value.value_name_id();
            let name = f
                .localized_strings(nameid)
                .next()
                .ok_or(CheckError::Error(
                    "Name reference in STAT table not found in name table".to_string(),
                ))?
                .chars()
                .collect::<String>();
            match axis_value {
                AxisValue::Format1(av) => {
                    let axis_tag = axes
                        .get(av.axis_index() as usize)
                        .ok_or(CheckError::Error(
                            "Axis not found in STAT table".to_string(),
                        ))?
                        .to_string();
                    if !AXES_TO_CHECK.contains(&axis_tag.as_str()) {
                        continue;
                    }
                    res.insert(
                        (axis_tag, name),
                        SimpleAxisValue {
                            flags: av.flags().bits(),
                            value: av.value().to_f32(),
                            linked_value: None,
                        },
                    );
                }
                AxisValue::Format2(av) => {
                    let axis_tag = axes
                        .get(av.axis_index() as usize)
                        .ok_or(CheckError::Error(
                            "Axis not found in STAT table".to_string(),
                        ))?
                        .to_string();
                    if !AXES_TO_CHECK.contains(&axis_tag.as_str()) {
                        continue;
                    }
                    res.insert(
                        (axis_tag, name),
                        SimpleAxisValue {
                            flags: av.flags().bits(),
                            value: av.nominal_value().to_f32(),
                            linked_value: None,
                        },
                    );
                }
                AxisValue::Format3(av) => {
                    let axis_tag = axes
                        .get(av.axis_index() as usize)
                        .ok_or(CheckError::Error(
                            "Axis not found in STAT table".to_string(),
                        ))?
                        .to_string();
                    if !AXES_TO_CHECK.contains(&axis_tag.as_str()) {
                        continue;
                    }
                    res.insert(
                        (axis_tag, name),
                        SimpleAxisValue {
                            flags: av.flags().bits(),
                            value: av.value().to_f32(),
                            linked_value: Some(av.linked_value().to_f32()),
                        },
                    );
                }
                AxisValue::Format4(_) => continue,
            }
        }
    }
    Ok(res)
}
#[check(
    id = "googlefonts/STAT/compulsory_axis_values",
    rationale = "
        
        Check a font's STAT table contains compulsory Axis Values which exist
        in the Google Fonts Axis Registry.

        We cannot determine what Axis Values the user will set for axes such as
        opsz, GRAD since these axes are unique for each font so we'll skip them.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/3800",
    title = "Check a font's STAT table contains compulsory Axis Values."
)]
fn compulsory_axis_values(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    skip!(
        !f.is_variable_font(),
        "not-variable",
        "Font is not a variable font."
    );
    let font_axis_values = stat_axis_values(&f.font())?;
    // XXX this should probably involve siblings
    let expected_binary = build_expected_font(&f, &[])?;
    let expected_font = FontRef::new(expected_binary.as_slice())?;
    let expected_axis_values = stat_axis_values(&expected_font)?;
    let rows: Vec<Vec<String>> = font_axis_values
        .keys()
        .chain(expected_axis_values.keys())
        .sorted()
        .unique()
        .map(|(axis, name)| {
            let current = font_axis_values.get(&(axis.clone(), name.clone()));
            let expected = expected_axis_values.get(&(axis.clone(), name.clone()));
            vec![
                name.to_string(),
                axis.to_string(),
                // current value
                current.map_or("N/A".to_string(), |v| v.value.to_string()),
                // expected value
                expected.map_or("N/A".to_string(), |v| v.value.to_string()),
                // current flags
                current.map_or("N/A".to_string(), |v| v.flags.to_string()),
                // expected flags
                expected.map_or("N/A".to_string(), |v| v.flags.to_string()),
                // current linked value
                current
                    .and_then(|v| v.linked_value)
                    .map_or("N/A".to_string(), |v| v.to_string()),
                // expected linked value
                expected
                    .and_then(|v| v.linked_value)
                    .map_or("N/A".to_string(), |v| v.to_string()),
            ]
        })
        .collect();
    let missing_italic_av = !rows.iter().any(|r| r[0].contains("Italic"));
    let mut table = MarkdownTable::new(rows);
    table.with_headings(vec![
        Heading::new("Name".to_string(), None),
        Heading::new("Axis".to_string(), None),
        Heading::new("Current Value".to_string(), None),
        Heading::new("Expected Value".to_string(), None),
        Heading::new("Current Flags".to_string(), None),
        Heading::new("Expected Flags".to_string(), None),
        Heading::new("Current Linked Value".to_string(), None),
        Heading::new("Expected Linked Value".to_string(), None),
    ]);
    let is_italic = f
        .font()
        .fvar()?
        .axes()?
        .iter()
        .any(|a| a.axis_tag() == "ital");
    if is_italic && missing_italic_av {
        problems.push(Status::fail(
            "missing-ital-axis-values",
            "Italic Axis Value missing.",
        ));
    }
    if font_axis_values != expected_axis_values {
        problems.push(Status::fail(
            "bad-axis-values",
            &format!(
                "Compulsory STAT Axis Values are incorrect:\n\n{}\n\n",
                table.to_string()
            ),
        ));
    }
    return_result(problems)
}
