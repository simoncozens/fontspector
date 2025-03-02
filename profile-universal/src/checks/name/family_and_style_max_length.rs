use std::collections::HashMap;

use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::{tables::name::Name, types::NameId, TableProvider};

fn strip_ribbi(name: &str) -> String {
    name.replace(" Regular", "")
        .replace(" Bold Italic", "")
        .replace(" Bold", "")
        .replace(" Italic", "")
}

// We are matching name entries with the same platform/language/encoding, so we
// need to use low-level APIS here
fn low_level_names(name: &Name<'_>, name_id: NameId) -> HashMap<(u16, u16, u16), String> {
    name.name_record()
        .iter()
        .filter(|r| r.name_id() == name_id)
        .map(|r| {
            (
                (r.platform_id(), r.encoding_id(), r.language_id()), // key
                r.string(name.string_data())
                    .map(|ns| ns.chars().collect::<String>())
                    .unwrap_or("".to_string()), // value
            )
        })
        .collect()
}

#[check(
    id = "name/family_and_style_max_length",
    rationale = "
        This check ensures that the length of name table entries is not
        too long, as this causes problems in some environments.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/1488",
    proposal = "https://github.com/fonttools/fontbakery/issues/2179",
    title = "Combined length of family and style must not exceed 32 characters."
)]
fn family_and_style_max_length(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    if f.get_name_entry_strings(NameId::FULL_NAME)
        .any(|name| strip_ribbi(&name).len() > 32)
    {
        problems.push(Status::fail(
            "nameid4-too-long",
            "Name ID 4 'Full Font Name' exceeds 32 characters. This has been found to cause problems with the dropdown menu in old versions of Microsoft Word as well as shaping issues for some accented letters in Microsoft Word on Windows 10 and 11.",
        ));
    }
    if f.get_name_entry_strings(NameId::POSTSCRIPT_NAME)
        .any(|name| name.len() > 27)
    {
        problems.push(Status::warn(
            "nameid6-too-long",
            "Name ID 6 'PostScript Name' exceeds 27 characters. This has been found to cause problems with PostScript printers, especially on Mac platforms.",
        ));
    }
    let name = f.font().name()?;
    let typo_family_names: HashMap<(u16, u16, u16), String> =
        low_level_names(&name, NameId::TYPOGRAPHIC_FAMILY_NAME);
    let family_names: HashMap<(u16, u16, u16), String> =
        low_level_names(&name, NameId::FAMILY_NAME);

    if f.has_table(b"fvar") {
        for instance in f.font().fvar()?.instances()?.iter().flatten() {
            for instance_name in f.get_name_entry_strings(instance.subfamily_name_id) {
                for (key, string) in family_names.iter() {
                    // Use typo if present, nameid=1 otherwise
                    let family_name = typo_family_names.get(key).unwrap_or(string);
                    let full_instance_name = format!("{} {}", family_name, instance_name);
                    if full_instance_name.len() > 32 {
                        problems.push(Status::fail(
                        "instance-too-long",
                        &format!(
                            "Variable font instance name '{}' formed by space-separated concatenation of font family name (nameID {}) and instance subfamily nameID {} exceeds 32 characters.\n\nThis has been found to cause shaping issues for some accented letters in Microsoft Word on Windows 10 and 11.",
                            full_instance_name,
                            NameId::FAMILY_NAME,
                            instance_name
                        ),
                    ));
                    }
                }
            }
        }
    }

    return_result(problems)
}
