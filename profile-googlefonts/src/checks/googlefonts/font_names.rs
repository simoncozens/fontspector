use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert, TestFont};
use markdown_table::{Heading, MarkdownTable};
use skrifa::string::StringId;

use crate::utils::build_expected_font;

const NAME_IDS: [(StringId, &str); 6] = [
    (StringId::FAMILY_NAME, "Family Name"),
    (StringId::SUBFAMILY_NAME, "Subfamily Name"),
    (StringId::FULL_NAME, "Full Name"),
    (StringId::POSTSCRIPT_NAME, "Postscript Name"),
    (StringId::TYPOGRAPHIC_FAMILY_NAME, "Typographic Family Name"),
    (
        StringId::TYPOGRAPHIC_SUBFAMILY_NAME,
        "Typographic Subfamily Name",
    ),
];
#[check(
    id = "googlefonts/font_names",
    rationale = "
        
        Google Fonts has several rules which need to be adhered to when
        setting a font's name table. Please read:
        https://googlefonts.github.io/gf-guide/statics.html#supported-styles
        https://googlefonts.github.io/gf-guide/statics.html#style-linking
        https://googlefonts.github.io/gf-guide/statics.html#unsupported-styles
        https://googlefonts.github.io/gf-guide/statics.html#single-weight-families
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/3800",
    title = "Check font names are correct"
)]
fn font_names(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    if f.has_axis("MORF") {
        return Ok(Status::just_one_warn("morf-axis",
            "Font has a Morph axis. This check only works on fonts that have a wght axis. Since users can define their own stylenames for Morph families, please manually check that the family works on major platforms. You can use Agu Display as a reference."
        ));
    }

    let expected_font_data = build_expected_font(&f, &[])?;
    let expected_font = TestFont::new_from_data(&t.filename, &expected_font_data)
        .map_err(|e| CheckError::Error(format!("Couldn't build expected font from data: {}", e)))?;
    let mut ok = true;
    let mut rows = vec![];
    for &(name_id, name) in NAME_IDS.iter() {
        let current = f.get_best_name(&[name_id]).unwrap_or("N/A".to_string());
        let expected = expected_font
            .get_best_name(&[name_id])
            .unwrap_or("N/A".to_string());

        let mut row = vec![name.to_string()];

        if name_id == StringId::FULL_NAME
            && expected.contains(" Regular")
            && current == expected.replace(" Regular", "")
        {
            problems.push(Status::warn(
                "lacks-regular",
                "Regular missing from full name",
            ));
        }
        if current != expected {
            row.push(format!("**{}**", current));
            row.push(format!("**{}**", expected));
            ok = false;
        } else {
            row.push(current);
            row.push(expected);
        }
        rows.push(row);
    }
    let mut md_table = MarkdownTable::new(rows);
    md_table.with_headings(vec![
        Heading::new("Name".to_string(), None),
        Heading::new("Current".to_string(), None),
        Heading::new("Expected".to_string(), None),
    ]);
    if !ok {
        problems.push(Status::fail(
            "bad-names",
            &format!(
                "Font names are incorrect:\n\n{}",
                md_table.as_markdown().map_err(|_| CheckError::Error(
                    "Can't happen (table creation failed)".to_string()
                ))?
            ),
        ));
    }
    return_result(problems)
}
