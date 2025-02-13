use counter::Counter;

use crate::checks::googlefonts::metadata::family_proto;
use fontspector_checkapi::prelude::*;

fn get_primary_script(ttFont: str) -> str {
    /*
        Zinh: "Inherited"
        Zyyy: "Common"
        Zzzz: "Unknown"
    */
    let most_common = ttFont
        .getBestCmap()
        .keys()
        .map(|c| unicodedata.script_extension(chr(c)))
        .filter(|script| !["Zinh", "Zyyy", "Zzzz"].contains(script))
        .collect::<Counter<_>>().most_common_ordered()[0];
    
    return most_common;
}

const SIBLINGS = [
    ["Kore", "Hang"],
    ["Jpan", "Hani", "Hant", "Hans"],
    ["Hira", "Kana"],
];

fn is_sibling_script(target: str, guessed: str) {
    return SIBLINGS.any(|family| family.contains(guessed) && family.contains(target));
}

fn get_sibling_scripts(target: str) -> Vec<Vec<str>> {
    return SIBLINGS.filter(|family| family.contains(target)).collect();
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
    let mut problems: Vec<Status> = vec![];
    let mdpb = c
        .get_file("METADATA.pb")
        .ok_or_else(|| CheckError::skip("no-mdpb", "No METADATA.pb file found"))?;
    let msg = family_proto(mdpb)?;
    let ttFont = c.get_file(msg.fonts[0]).ok_or_else(|| CheckError::skip("no-font-file", "No font binary file found"))?;
    let metadata_primary_script = msg.primary_script();
    let guessed_primary_script = get_primary_script(ttFont);
    let mut message = "";

    if guessed_primary_script != "Latn" {
        /* family_metadata.primary_script is empty but should be set */
        if [None, ""].contains(metadata_primary_script) {
            message += (
                &format!(
                    "METADATA.pb: primary_script field should be '{}' but is missing.",
                    guessed_primary_script,
                )
            );
            sibling_scripts = get_sibling_scripts(guessed_primary_script);
            if !sibling_scripts.is_empty() {
                sibling_scripts = ", ".join(sibling_scripts[0]);
                message += &format!(
                    "\nMake sure that '{}' is actually the correct one (out of {}).",
                    guessed_primary_script,
                    sibling_scripts,
                );
            }
            problems.push(Status::warn("missing-primary-script", message));
        }

        /* family_metadata.primary_script is set
           but it's not the same as guessed_primary_script */
        if (
            ![None, ""].contains(metadata_primary_script)
            && metadata_primary_script != guessed_primary_script
            && is_sibling_script(metadata_primary_script, guessed_primary_script) is None
        ) {
            problems.push(Status::warn(
                "wrong-primary-script",
                &format!(
                    "METADATA.pb: primary_script is '{}'\nIt should instead be '{}'.",
                    metadata_primary_script,
                    guessed_primary_script,
                ),
            ));
        }
    }
    return_result(problems)
}
