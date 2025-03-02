use fontspector_checkapi::{prelude::*, FileTypeConvert};
use hashbrown::HashMap;
use skrifa::string::StringId;

use crate::checks::googlefonts::metadata::family_proto;
use crate::constants::EXPECTED_COPYRIGHT_PATTERN;

#[check(
    id = "googlefonts/font_copyright",
    rationale = "
        
        This check aims at ensuring a uniform and legally accurate copyright statement
        on the name table entries and METADATA.pb files of font files across the Google
        Fonts library.

        We also check that the copyright field in METADATA.pb matches the
        contents of the name table nameID 0 (Copyright), and that the copyright
        notice within the METADATA.pb file is not too long; if it is more than 500
        characters, this may be an indication that either a full license or the
        font's description has been included in this field by mistake.

    
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
    proposal = "https://github.com/fonttools/fontbakery/pull/2383",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Copyright notices match canonical pattern in fonts",
    implementation = "all"
)]
fn font_copyright(c: &TestableCollection, context: &Context) -> CheckFnResult {
    let fonts = TTF.from_collection(c);
    let mut problems = vec![];
    let mdpb = c
        .get_file("METADATA.pb")
        .map(|x: &Testable| family_proto(x))
        .transpose()?;
    let mdpb_copyrights = mdpb
        .map(|proto| {
            proto
                .fonts
                .iter()
                .map(|f| (f.filename().to_string(), f.copyright().to_string()))
                .collect::<HashMap<String, _>>()
        })
        .unwrap_or_default();

    for font in fonts {
        let basename = font
            .filename
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or_default();
        let filename = font.filename.as_os_str().to_str().unwrap_or_default();
        let mut copyrights = vec![];
        font.get_name_entry_strings(StringId::COPYRIGHT_NOTICE)
            .for_each(|x| copyrights.push(("Name Table entry", x)));
        if let Some(md_copyright) = mdpb_copyrights.get(basename) {
            copyrights.push(("METADATA.pb", md_copyright.to_string()));
        }
        println!("{}: {:?}", filename, copyrights);

        let mut copyright_sources: HashMap<String, Vec<&str>> = HashMap::new();
        for (source, string) in copyrights {
            let string = string.to_lowercase();
            if !EXPECTED_COPYRIGHT_PATTERN.is_match(&string) {
                problems.push(Status::fail(
                "bad-notice-format",
                &format!(
                  "{}: Copyright notices should match a pattern similar to:\n\n\"Copyright 2020 The Familyname Project Authors (git url)\"\n\nBut instead we have got:\n\n\"{}\"",
                  source, string
                )
              )
            );
            }
            if string.len() > 500 {
                problems.push(Status::fail(
                "max-length",
                &format!(
                  "{}: The length of the following copyright notice ({}) exceeds 500 chars:\n\n\"{}\"",
                  source, string.len(), string
                )
              )
            );
            }
            let entry = copyright_sources.entry(string);
            entry.or_default().push(source);
        }
        if copyright_sources.len() > 1 {
            problems.push(Status::fail(
                "mismatch",
                &format!("Copyright notices differ between name table entries and METADATA.pb. The following entries were found:\n{}",
                bullet_list(
                  context,
                  copyright_sources.iter().map(|(a, b)| format!("{}: {:?}", a, b))
              )
            )
          ))
        }
    }

    return_result(problems)
}
