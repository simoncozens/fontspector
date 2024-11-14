use std::collections::HashMap;

use fontspector_checkapi::{prelude::*, FileTypeConvert};
use read_fonts::TableProvider;

#[check(
    id = "family/vertical_metrics",
    rationale = "
        We want all fonts within a family to have the same vertical metrics so
        their line spacing is consistent across the family.
    ",
    implementation = "all",
    title = "Each font in a family must have the same set of vertical metrics values.",
    proposal = "https://github.com/fonttools/fontbakery/issues/1487"
)]
fn family_vertical_metrics(c: &TestableCollection, context: &Context) -> CheckFnResult {
    let fonts = TTF.from_collection(c);
    let mut problems = vec![];
    let mut metrics = HashMap::new();
    let mut font_names = vec![];
    for font in fonts.iter() {
        let basename = font
            .filename
            .file_name()
            .and_then(|x| x.to_str())
            .map(|x| x.to_string())
            .unwrap_or("A font".to_string());
        if let Ok(os2) = font.font().os2() {
            metrics
                .entry("sTypoAscender")
                .or_insert_with(Vec::new)
                .push(os2.s_typo_ascender());
            metrics
                .entry("sTypoDescender")
                .or_insert_with(Vec::new)
                .push(os2.s_typo_descender());
            metrics
                .entry("sTypoLineGap")
                .or_insert_with(Vec::new)
                .push(os2.s_typo_line_gap());
            metrics
                .entry("usWinAscent")
                .or_insert_with(Vec::new)
                .push(os2.us_win_ascent() as i16);
            metrics
                .entry("usWinDescent")
                .or_insert_with(Vec::new)
                .push(os2.us_win_descent() as i16);
        } else {
            problems.push(Status::fail(
                "lacks-OS/2",
                &format!("{} lacks an 'OS/2' table.", basename),
            ));
        }
        if let Ok(hhea) = font.font().hhea() {
            metrics
                .entry("ascent")
                .or_insert_with(Vec::new)
                .push(hhea.ascender().to_i16());
            metrics
                .entry("descent")
                .or_insert_with(Vec::new)
                .push(hhea.descender().to_i16());
            metrics
                .entry("lineGap")
                .or_insert_with(Vec::new)
                .push(hhea.line_gap().to_i16());
        } else {
            problems.push(Status::fail(
                "lacks-hhea",
                &format!("{} lacks a 'hhea' table.", basename),
            ));
        }
        font_names.push(basename);
    }
    if !problems.is_empty() {
        return return_result(problems);
    }

    for (key, values) in metrics.iter() {
        let all_the_same = values.iter().all(|a| a == &values[0]);
        if !all_the_same {
            problems.push(Status::fail(
                &format!("{}-mismatch", key),
                &format!(
                    "{} is not the same across the family: {}",
                    key,
                    bullet_list(
                        context,
                        values
                            .iter()
                            .zip(font_names.iter())
                            .map(|(a, b)| format!("{}: {}", b, a))
                    )
                ),
            ))
        }
    }

    return_result(problems)
}
