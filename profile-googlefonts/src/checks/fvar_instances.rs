use std::vec;

use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert, TestFont};
use indexmap::{IndexMap, IndexSet};
use markdown_table::{Heading, MarkdownTable};

use crate::utils::build_expected_font;

#[check(
    id = "googlefonts/fvar_instances",
    rationale = "
        
        Check a font's fvar instance coordinates comply with our guidelines:
        https://googlefonts.github.io/gf-guide/variable.html#fvar-instances

        This check is skipped for fonts that have a Morph (MORF) axis
        since we allow users to define their own custom instances.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/3800",
    title = "Check variable font instances"
)]
fn googlefonts_fvar_instances(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    skip!(f.has_axis("MORF"), "has-morf", "Font has a MORF axis");
    let expected_font_data = build_expected_font(&f, &[])?;
    let expected_font = TestFont::new_from_data(&t.filename, &expected_font_data)
        .map_err(|e| CheckError::Error(format!("Couldn't build expected font from data: {}", e)))?;
    let instances: IndexMap<String, _> = f.named_instances().collect();
    let expected_instances: IndexMap<String, _> = expected_font.named_instances().collect();
    let mut table = vec![];
    for name in instances.keys().chain(expected_instances.keys()) {
        let mut row = IndexMap::new();
        row.insert("Name".to_string(), name.to_string());
        if let Some(font_instance) = instances.get(name) {
            row.insert(
                "current".to_string(),
                font_instance
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }
        if let Some(expected_instance) = expected_instances.get(name) {
            row.insert(
                "expected".to_string(),
                expected_instance
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }
        table.push(row);
    }
    table.sort_by(|a: &IndexMap<String, String>, b| a.get("expected").cmp(&b.get("expected")));
    let expected_names: IndexSet<_> = expected_instances.keys().collect();
    let current_names: IndexSet<_> = instances.keys().collect();
    let missing_names = expected_names
        .difference(&current_names)
        .collect::<Vec<_>>();
    let new_names = current_names
        .difference(&expected_names)
        .collect::<Vec<_>>();
    let same_names = current_names
        .intersection(&expected_names)
        .collect::<Vec<_>>();
    let wght_wrong = expected_instances.values().all(|i| i.contains_key("wght"))
        && same_names
            .iter()
            .any(|i| instances[**i]["wght"] != expected_instances[**i]["wght"]);
    let mut md_table = MarkdownTable::new(
        table
            .iter()
            .map(|ix| {
                vec![
                    ix.get("Name").map_or("Unknown", |s| s.as_ref()),
                    ix.get("current").map_or("N/A", |s| s.as_ref()),
                    ix.get("expected").map_or("N/A", |s| s.as_ref()),
                ]
            })
            .collect(),
    );
    md_table.with_headings(vec![
        Heading::new("Name".to_string(), None),
        Heading::new("current".to_string(), None),
        Heading::new("expected".to_string(), None),
    ]);
    if wght_wrong || !missing_names.is_empty() || !new_names.is_empty() {
        let mut hints = vec![];
        if !missing_names.is_empty() {
            hints.push("- Add missing instances");
        }
        if !new_names.is_empty() {
            hints.push("- Delete additional instances");
        }
        if wght_wrong {
            hints.push("- wght coordinates are wrong for some instances");
        }
        problems.push(Status::fail(
            "bad-fvar-instances",
            &format!(
                "fvar instances are incorrect:\n\n{}\n\n{}\n\n",
                hints.join("\n"),
                md_table.as_markdown().map_err(|_| CheckError::Error(
                    "Can't happen (table creation failed)".to_string()
                ))?
            ),
        ));
    } else if same_names
        .into_iter()
        .any(|i| instances[*i] != expected_instances[*i])
    {
        problems.push(Status::warn(
            "suspicious-fvar-coords",
            &format!(
                "fvar instance coordinates for non-wght axes are not the same as the fvar defaults. This may be intentional so please check with the font author:\n\n{}\n\n",
                md_table.as_markdown().map_err(|_| CheckError::Error("Can't happen (table creation failed)".to_string()))?
            ),
        ));
    }
    return_result(problems)
}
