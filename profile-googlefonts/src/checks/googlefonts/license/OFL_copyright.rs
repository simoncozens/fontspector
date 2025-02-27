use crate::constants::EXPECTED_COPYRIGHT_PATTERN;
use fontspector_checkapi::prelude::*;

#[check(
    id = "googlefonts/license/OFL_copyright",
    rationale = "
        
        An OFL.txt file's first line should be the font copyright.

    
        The expected pattern for the copyright string adheres to the following rules:

        * It must say \"Copyright\" followed by a 4 digit year (optionally followed by
          a hyphen and another 4 digit year)

        * Additional years or year ranges are also valid.

        * An optional comma can be placed here.

        * Then it must say \"The <familyname> Project Authors\" and, within parentheses,
          a URL for a git repository must be provided. But we have an exception
          for the fonts from the Noto project, that simply have
          \"google llc. all rights reserved\" here.

        * The check is case insensitive and does not validate whether the familyname
          is correct, even though we'd obviously expect it to be.


        Here is an example of a valid copyright string:

        \"Copyright 2017 The Archivo Black Project Authors
         (https://github.com/Omnibus-Type/ArchivoBlack)\"

    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/2764",
    title = "Check license file has good copyright string.",
    applies_to = "LICENSE"
)]
fn OFL_copyright(t: &Testable, _context: &Context) -> CheckFnResult {
    let license_contents = String::from_utf8(t.contents.clone())
        .map_err(|e| CheckError::Error(format!("OFL.txt is not valid UTF-8: {:?}", e)))?
        .trim()
        .split("\n")
        .next()
        .ok_or(CheckError::Error("OFL.txt is empty".to_string()))?
        .to_lowercase();
    if !EXPECTED_COPYRIGHT_PATTERN.is_match(&license_contents) {
        return Ok(Status::just_one_fail("bad-format",
          &format!("First line in license file is:\n\n\"{}\"\n\nwhich does not match the expected format, similar to:\n\n\"Copyright 2022 The Familyname Project Authors (git url)\"",
          license_contents)));
    }
    Ok(Status::just_one_pass())
}
