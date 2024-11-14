use std::collections::HashMap;

use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use read_fonts::{tables::stat::AxisValue, TableProvider};
use skrifa::Tag;

#[check(
    id = "STAT_in_statics",
    rationale = "
        Adobe feature syntax allows for the definition of a STAT table. Fonts built
        with a hand-coded STAT table in feature syntax may be built either as static
        or variable, but will end up with the same STAT table.

        This is a problem, because a STAT table which works on variable fonts
        will not be appropriate for static instances. The examples in the OpenType spec
        of non-variable fonts with a STAT table show that the table entries must be
        restricted to those entries which refer to the static font's position in
        the designspace. i.e. a Regular weight static should only have the following
        entry for the weight axis:

        ```
        <AxisIndex value=\"0\"/>
        <Flags value=\"2\"/>  <!-- ElidableAxisValueName -->
        <ValueNameID value=\"265\"/>  <!-- Regular -->
        <Value value=\"400.0\"/>
        ```

        However, if the STAT table intended for a variable font is compiled into a
        static, it will have many entries for this axis. In this case, Windows will
        read the first entry only, causing all instances to report themselves
        as \"Thin Condensed\".
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4149",
    title = "Checking STAT table entries in static fonts."
)]
fn stat_in_statics(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.has_table(b"STAT"), "no-stat", "No STAT table.");
    skip!(
        f.is_variable_font(),
        "variable-font",
        "This is a variable font."
    );
    let mut counter: HashMap<Tag, usize> = HashMap::new();
    let stat = f.font().stat()?;
    let axes = stat.design_axes()?;
    if let Some(Ok(subtable)) = stat.offset_to_axis_values() {
        for axis_value in subtable.axis_values().iter().flatten() {
            match axis_value {
                AxisValue::Format1(v) => {
                    let axis_tag = axes
                        .get(v.axis_index() as usize)
                        .ok_or(CheckError::Error(
                            "Bad axis index in avis value record".to_string(),
                        ))?
                        .axis_tag();
                    *counter.entry(axis_tag).or_insert(0) += 1;
                }
                AxisValue::Format2(v) => {
                    let axis_tag = axes
                        .get(v.axis_index() as usize)
                        .ok_or(CheckError::Error(
                            "Bad axis index in avis value record".to_string(),
                        ))?
                        .axis_tag();
                    *counter.entry(axis_tag).or_insert(0) += 1;
                }
                AxisValue::Format3(v) => {
                    let axis_tag = axes
                        .get(v.axis_index() as usize)
                        .ok_or(CheckError::Error(
                            "Bad axis index in avis value record".to_string(),
                        ))?
                        .axis_tag();
                    *counter.entry(axis_tag).or_insert(0) += 1;
                }
                AxisValue::Format4(v) => {
                    for axis_value in v.axis_values() {
                        let axis_tag = axes
                            .get(axis_value.axis_index() as usize)
                            .ok_or(CheckError::Error(
                                "Bad axis index in avis value record".to_string(),
                            ))?
                            .axis_tag();
                        *counter.entry(axis_tag).or_insert(0) += 1;
                    }
                }
            }
        }
    }
    let problems = counter
        .into_iter()
        .filter(|&(_axis, count)| { count > 1 })
        .map(|(axis, count)| { Status::fail(
                "multiple-STAT-entries",
                &format!("The STAT table has more than a single entry for the '{}' axis ({}) on this static font which will causes problems on Windows.",
                axis,
                count)
            )}).collect();
    return_result(problems)
}
