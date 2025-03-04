use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert, TestFont};
use hashbrown::HashMap;
use unicode_script::UnicodeScript;

use crate::checks::googlefonts::metadata::family_proto;

fn get_primary_script(font: &TestFont, context: &Context) -> String {
    let mut script_count = HashMap::new();
    for c in font
        .codepoints(Some(context))
        .into_iter()
        .filter_map(char::from_u32)
    {
        for script in c.script_extension().iter() {
            let name = script.short_name();
            if !["Zinh", "Zyyy", "Zzzz"].contains(&name) {
                *script_count.entry(name).or_insert(0) += 1;
            }
        }
    }
    let most_common = script_count.iter().max_by_key(|(_, &count)| count);
    if let Some((script, _)) = most_common {
        script.to_string()
    } else {
        "Latn".to_string()
    }
}

fn siblings(script: &str) -> Option<Vec<&'static str>> {
    match script {
        "Kore" => Some(vec!["Kore", "Hang"]),
        "Jpan" => Some(vec!["Jpan", "Hani", "Hant", "Hans"]),
        "Hira" => Some(vec!["Hira", "Kana"]),
        _ => None,
    }
}

fn is_sibling_script(script1: &str, script2: &str) -> bool {
    siblings(script1).unwrap_or_default().contains(&script2)
}

#[check(
    id = "googlefonts/metadata/primary_script",
    rationale = "
        
        Try to guess font's primary script and see if that's set in METADATA.pb.
        This is an educated guess based on the number of glyphs per script in the font
        and should be taken with caution.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4109",
    title = "METADATA.pb: Check for primary_script",
    implementation = "all"
)]
fn primary_script(c: &TestableCollection, context: &Context) -> CheckFnResult {
    let mut problems = vec![];
    let mdpb = c
        .get_file("METADATA.pb")
        .ok_or_else(|| CheckError::skip("no-mdpb", "No METADATA.pb file found"))?;
    let msg = family_proto(mdpb)?;
    let fonts = msg
        .fonts
        .iter()
        .flat_map(|f| f.filename.as_ref())
        .flat_map(|f| c.get_file(f))
        .collect::<Vec<&Testable>>();
    let metadata_primary_script = msg.primary_script();
    if fonts.is_empty() {
        skip!("no-fonts", "No font files found in METADATA.pb");
    }
    for font in fonts {
        let f = testfont!(font);
        let guessed_primary_script = get_primary_script(&f, context);
        if guessed_primary_script == "Latn" {
            continue;
        }
        if metadata_primary_script.is_empty() {
            let mut message = format!(
                "METADATA.pb: primary_script field should be '{}' but is missing.",
                guessed_primary_script
            );
            if let Some(sibling_scripts) = siblings(&guessed_primary_script) {
                let sibling_scripts = sibling_scripts.join(", ");
                message += &format!(
                    "\nMake sure that '{}' is actually the correct one (out of {}).",
                    guessed_primary_script, sibling_scripts
                );
            }
            problems.push(Status::warn("missing-primary-script", &message));
        } else if metadata_primary_script != guessed_primary_script
            && !is_sibling_script(metadata_primary_script, &guessed_primary_script)
        {
            problems.push(Status::warn(
                "wrong-primary-script",
                &format!(
                    "METADATA.pb: primary_script is '{}' but should be '{}'.",
                    metadata_primary_script, guessed_primary_script
                ),
            ));
        }
    }
    return_result(problems)
}
