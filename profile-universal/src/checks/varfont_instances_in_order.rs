use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use itertools::Itertools;
use std::collections::HashMap;

#[check(
    id = "varfont/instances_in_order",
    rationale = "
        Ensure that the fvar table instances are in ascending order of weight.
        Some software, such as Canva, displays the instances in the order they
        are defined in the fvar table, which can lead to confusion if the
        instances are not in order of weight.
    ",
    proposal = "https://github.com/googlefonts/fontbakery/issues/3334",
    title = "Ensure the font's instances are in the correct order."
)]
fn varfont_instances_in_order(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.has_axis("wght"), "no-wght", "Font has no weight axis");
    let mut problems = vec![];
    let mut sublists: Vec<Vec<HashMap<String, f32>>> = vec![vec![]];
    let coords = f.named_instances().map(|(_name, coords)| coords);
    let mut last_non_wght = None;

    // Partition into sub-lists based on the other axes values.
    // e.g. "Thin Regular", "Bold Regular", "Thin Condensed", "Bold Condensed"
    // becomes [ ["Thin Regular", "Bold Regular"], ["Thin Condensed", "Bold Condensed"] ]
    for coord in coords {
        let mut non_wght = coord.clone();
        non_wght.remove("wght");
        if Some(&non_wght) != last_non_wght.as_ref() {
            sublists.push(vec![]);
            last_non_wght = Some(non_wght);
        }
        #[allow(clippy::unwrap_used)] // We know the last element exists
        sublists.last_mut().unwrap().push(coord);
    }

    // Check if the instances are in order of weight
    for sublist in sublists {
        let wght_values: Vec<&f32> = sublist.iter().flat_map(|i| i.get("wght")).collect();
        if !wght_values.iter().is_sorted() {
            problems.push(Status::fail(
                "instances-not-in-order",
                &format!(
                    "The fvar table instances are not in ascending order of weight:\n{}",
                    bullet_list(
                        context,
                        sublist.iter().map(|coords| coords
                            .iter()
                            .map(|(k, v)| format!("{}={}", k, v))
                            .join(", "))
                    ),
                ),
            ));
        }
    }
    return_result(problems)
}
