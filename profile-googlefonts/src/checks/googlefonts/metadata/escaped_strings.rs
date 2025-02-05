use fontspector_checkapi::prelude::*;
use std::fs::read_to_string;

#[check(
    id = "googlefonts/metadata/escaped_strings",
    rationale = "
        
        In some cases we've seen designer names and other fields with escaped strings
        in METADATA files (such as \"Juli\\303\\241n\").

        Nowadays the strings can be full unicode strings (such as \"Julián\") and do
        not need escaping.

        Escaping quotes or double-quotes is fine, though.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/2932",
    title = "Ensure METADATA.pb does not use escaped strings.",
    implementation = "all"
)]
fn escaped_strings(c: &TestableCollection, _context: &Context) -> CheckFnResult {
    let mdpb = c
        .get_file("METADATA.pb")
        .ok_or_else(|| CheckError::skip("no-mdpb", "No METADATA.pb file found"))?;
    let mut problems = vec![];

    for line in read_to_string(mdpb.filename.clone()).unwrap().lines() {
        // Escaped quotes are fine!
        // What we're really interested in detecting are things like
        // "Juli\303\241n" instead of "Julián"
        let mut line_string = line.to_string();
        line_string = line_string.replace("\\'", "").replace("\\\"", "");
        for quote_char in vec!["'", "\""] {
            let segments = line_string.split(quote_char).collect::<Vec<&str>>();
            if segments.len() >= 3 {
                let a_string = segments[1];
                if a_string.contains("\\") {
                    problems.push(Status::fail(
                        "escaped-strings",
                        format!(
                            "Found escaped chars at '{}'. Please use an unicode string instead.",
                            a_string
                        )
                        .as_str(),
                    ));
                }
            }
        }
    }
    return_result(problems)
}
