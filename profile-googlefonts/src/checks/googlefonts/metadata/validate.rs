use std::path::Path;

use crate::checks::googlefonts::metadata::family_proto;
use chrono::prelude::*;
use fontspector_checkapi::prelude::*;
use hashbrown::HashSet;

fn weight_acceptable_suffixes(w: i32) -> Vec<&'static str> {
    match w {
        100 => vec!["Thin", "ThinItalic"],
        200 => vec!["ExtraLight", "ExtraLightItalic"],
        300 => vec!["Light", "LightItalic"],
        400 => vec!["Regular", "Italic"],
        500 => vec!["Medium", "MediumItalic"],
        600 => vec!["SemiBold", "SemiBoldItalic"],
        700 => vec!["Bold", "BoldItalic"],
        800 => vec!["ExtraBold", "ExtraBoldItalic"],
        900 => vec!["Black", "BlackItalic"],
        _ => vec![],
    }
}

const CATEGORY_HINTS: [(&str, &str); 11] = [
    ("Sans", "SANS_SERIF"),
    ("Grotesk", "SANS_SERIF"),
    ("Grotesque", "SANS_SERIF"),
    ("Serif", "SERIF"),
    ("Transitional", "SERIF"),
    ("Slab", "SERIF"),
    ("Old Style", "SERIF"),
    ("Garamond", "SERIF"),
    ("Display", "DISPLAY"),
    ("Hand", "HANDWRITING"),
    ("Script", "HANDWRITING"),
];

fn category_hints(family_name: &str) -> Option<&'static str> {
    for (component, inferred_category) in CATEGORY_HINTS.iter() {
        if family_name.contains(component) {
            return Some(inferred_category);
        }
    }
    None
}

fn clean_url(url: &str) -> String {
    let mut cleaned = url.trim().to_string();
    if cleaned.ends_with('/') {
        cleaned.pop();
    }
    if cleaned.ends_with("index.htm") {
        cleaned = cleaned.replace("index.htm", "");
    }
    if cleaned.ends_with("index.html") {
        cleaned = cleaned.replace("index.html", "");
    }
    cleaned
}

#[check(
    id = "googlefonts/metadata/validate",
    title = "Check METADATA.pb parses correctly",
    rationale = "
        The purpose of this check is to ensure that the METADATA.pb file is not
        malformed.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/2248",
    applies_to = "MDPB"
)]
fn validate(c: &Testable, _context: &Context) -> CheckFnResult {
    let msg = family_proto(c).map_err(|e| {
        CheckError::Error(format!("METADATA.pb is not a valid FamilyProto: {:?}", e))
    })?;
    let mut problems = vec![];
    if let Some(designer) = msg.designer.as_ref() {
        if designer.is_empty() {
            problems.push(Status::fail(
                "empty-designer",
                "Font designer field is empty.",
            ))
        }
        if designer.contains('/') {
            problems.push(Status::fail("slash",
                    &format!(
                    "Font designer field contains a forward slash '{}'. Please use commas to separate multiple names instead.",
                    designer
                )));
        }
    }

    // Check date added is YYYY-MM-DD
    if msg.date_added().is_empty() {
        problems.push(Status::error(Some("date-empty"), "Date added is empty"))
    }

    if msg
        .date_added
        .as_ref()
        .is_some_and(|da| NaiveDate::parse_from_str(da, "%Y-%m-%d").is_err())
    {
        problems.push(Status::error(
            Some("date-malformed"),
            "Date added is not in the format YYYY-MM-DD",
        ))
    }

    // Check category hints (googlefonts/metadata/category_hints)
    if let Some(inferred_category) = category_hints(msg.name()) {
        if !msg.category.contains(&inferred_category.to_string()) {
            problems.push(Status::warn(
                "inferred-category",
                &format!(
                    "Familyname seems to hint at \"{}\" category, but METADATA.pb declares it as \"{}\".",
                    inferred_category,
                    msg.category.join(", "),
                ),
            ));
        }
    }

    // Check minisite URL (googlefonts/metadata/minisite_url)
    if msg.minisite_url().is_empty() {
        problems.push(Status::info(
            "lacks-minisite-url",
            "Please consider adding a family.minisite_url entry.",
        ))
    } else {
        let expected_url = clean_url(msg.minisite_url());
        if msg.minisite_url() != expected_url {
            problems.push(Status::fail(
                "trailing-clutter",
                &format!(
                    "Please change minisite_url from {} to {}",
                    msg.minisite_url(),
                    expected_url
                ),
            ))
        }
    }

    for font in msg.fonts.iter() {
        // Check weight values are canonical (googlefonts/metadata/canonical_weight_value)
        if ![100, 200, 300, 400, 500, 600, 700, 800, 900].contains(&font.weight()) {
            problems.push(Status::fail(
                    "bad-weight",
                    &format!("In METADATA.pb, the weight for {} is declared as {}, which is not a multiple of 100 between 100 and 900.",
                        font.full_name(), font.weight()),
                ))
        }
        // skip variable fonts
        if !font.filename().contains("[") {
            let post_script_name = font.post_script_name();
            // Check weight values match post_script_name (googlefonts/metadata/match_weight_postscript)
            let weight_suffixes = weight_acceptable_suffixes(font.weight());
            if !weight_suffixes
                .iter()
                .any(|suffix| post_script_name.ends_with(suffix))
            {
                problems.push(Status::fail(
                        "mismatch",
                        &format!(
                            "METADATA.pb: Mismatch between postScriptName {} and and weight value ({}). The name must end with {}",
                            font.weight(),
                            post_script_name,
                            weight_suffixes.join(" or ")
                        ),
                ));
            }

            // Check font.filename matches font.post_script_name (googlefonts/metadata/match_filename_postscript)
            if let Some(basename) = Path::new(font.filename()).file_stem() {
                if post_script_name != basename {
                    problems.push(Status::fail(
                            "mismatch",
                            &format!(
                                "METADATA.pb font filename = \"{}\" does not match post_script_name=\"{}\".",
                                font.filename(),
                                post_script_name,
                            ),
                    ));
                }
            }
        }
        // Check font.fullname matches font.post_script_name (with non-alphabetic removed) (googlefonts/metadata/match_fullname_postscript)
        if font.full_name().replace(|c| !char::is_alphanumeric(c), "")
            != font
                .post_script_name()
                .replace(|c| !char::is_alphanumeric(c), "")
        {
            problems.push(Status::fail(
                "mismatch",
                &format!(
                    "METADATA.pb font fullname = \"{}\" does not match post_script_name=\"{}\".",
                    font.full_name(),
                    font.post_script_name(),
                ),
            ));
        }

        // Check font name is same as family name (googlefonts/metadata/match_name_familyname)
        if font.name() != msg.name() {
            problems.push(Status::fail(
                "mismatch",
                &format!(
                    "METADATA.pb: {}:\n Family name \"{}\" does not match font name: \"{}\"",
                    font.filename(),
                    msg.name(),
                    font.name(),
                ),
            ));
        }
    }

    // unique_full_name_values
    let full_names = msg
        .fonts
        .iter()
        .map(|f| f.full_name())
        .collect::<HashSet<_>>();
    if full_names.len() != msg.fonts.len() {
        problems.push(Status::fail(
            "duplicated",
            "Found duplicated \"full_name\" values in METADATA.pb fonts field.",
        ));
    }

    return_result(problems)
}
