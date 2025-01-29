use fontspector_checkapi::{prelude::*, FileTypeConvert};
use hashbrown::HashMap;
use read_fonts::TableProvider;

#[check(
    id = "googlefonts/STAT/axis_order",
    rationale = "
        
        This is (for now) a merely informative check to detect what's the axis ordering
        declared on the STAT table of fonts in the Google Fonts collection.

        We may later update this to enforce some unified axis ordering scheme,
        yet to be determined.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3049",
    title = "Check axis ordering on the STAT table.",
    implementation = "all"
)]
fn axis_order(c: &TestableCollection, _context: &Context) -> CheckFnResult {
    let fonts = TTF.from_collection(c);
    let mut no_stat = 0;
    let mut summary = HashMap::new();
    let mut results = vec![];
    for font in fonts.iter() {
        if let Ok(stat) = font.font().stat() {
            let mut axes = stat
                .design_axes()?
                .iter()
                .map(|axis| (axis.axis_tag(), axis.axis_ordering()))
                .collect::<Vec<_>>();
            axes.sort_by_key(|(_tag, ordering)| *ordering);
            let key = axes
                .iter()
                .map(|(tag, _)| tag.to_string())
                .collect::<Vec<_>>()
                .join("-");
            summary.entry(key).and_modify(|e| *e += 1).or_insert(1);
        } else {
            no_stat += 1;
            results.push(Status::skip(
                "missing-STAT",
                &format!(
                    "This font does not have a STAT table: {}",
                    font.filename.to_string_lossy()
                ),
            ));
        }
    }

    let percentage = if no_stat == 0 {
        "None".to_string()
    } else if no_stat == fonts.len() {
        "All".to_string()
    } else {
        format!("{:.2}%", 100.0 * no_stat as f32 / fonts.len() as f32)
    };

    if !summary.is_empty() {
        let report = summary
            .iter()
            .map(|(key, count)| format!("{}: {}", key, count))
            .collect::<Vec<_>>()
            .join("\n\t");
        results.push(Status::info(
            "summary",
            &format!(
                "{} of the fonts lack a STAT table.\n\n\tAnd these are the most common STAT axis orderings:\n\t{}",
                percentage, report
            ),
        ));
    } else {
        results.push(Status::info(
            "summary",
            &format!("{} of the fonts lack a STAT table.", percentage),
        ));
    }
    return_result(results)
}
