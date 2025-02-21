use std::path::Path;

use crate::checks::googlefonts::metadata::family_proto;
use chrono::prelude::*;
use fontspector_checkapi::prelude::*;

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
    if msg
        .date_added
        .as_ref()
        .is_some_and(|da| NaiveDate::parse_from_str(da, "%Y-%m-%d").is_err())
    {
        problems.push(Status::fail(
            "date-malformed",
            "Date added is not in the format YYYY-MM-DD",
        ))
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

    return_result(problems)
}
