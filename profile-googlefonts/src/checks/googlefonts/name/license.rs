use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;
use skrifa::string::StringId;

use std::collections::HashMap;

#[check(
    id = "googlefonts/name/license",
    rationale = "
        
        A known licensing description must be provided in the NameID 14
        (LICENSE DESCRIPTION) entries of the name table.

        The source of truth for this check (to determine which license is in use) is
        a file placed side-by-side to your font project including the licensing terms.

        Depending on the chosen license, one of the following string snippets is
        expected to be found on the NameID 13 (LICENSE DESCRIPTION) entries of the
        name table:

        - \"This Font Software is licensed under the SIL Open Font License, Version 1.1.
          This license is available with a FAQ at: openfontlicense.org\"

        - \"Licensed under the Apache License, Version 2.0\"

        - \"Licensed under the Ubuntu Font Licence 1.0.\"


        Currently accepted licenses are Apache or Open Font License. For a small set
        of legacy families the Ubuntu Font License may be acceptable as well.

        When in doubt, please choose OFL for new font projects.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Check copyright namerecords match license file."
)]
fn license(t: &Testable, _context: &Context) -> CheckFnResult {
    let placeholder_licensing_text: HashMap<&str, &str> = HashMap::from([
        ("UFL.txt", "Licensed under the Ubuntu Font Licence 1.0."),
        (
            "OFL.txt",
            "This Font Software is licensed under the SIL Open Font License, Version 1.1. This license is available with a FAQ at: https://openfontlicense.org",
        ),
        (
            "LICENSE.txt",
            "Licensed under the Apache License, Version 2.0",
        ),
    ]);

    let f = testfont!(t);
    let mut problems = vec![];
    let mut http_warn = false;

    // FIXME: This should probably come from a TestableCollection, I guess...
    let license_filename = "LICENSE.txt";

    let placeholder = placeholder_licensing_text[license_filename];
    let mut entry_found: bool = false;
    let name = f.font().name()?;
    let mut license_description = "".to_string();
    for record in name.name_record().iter() {
        let string = record.string(name.string_data())?;
        if record.name_id() == StringId::LICENSE_DESCRIPTION {
            license_description = string.to_string();
            entry_found = true;
            if license_description.contains("http://") {
                problems.push(Status::warn(
                    "http-in-description",
                    &format!(
                        "Please consider using HTTPS URLs at \
                              name table entry \
                              [plat={}, enc={}, name={}]",
                        record.platform_id(),
                        record.encoding_id(),
                        record.name_id()
                    ),
                ));
                license_description = license_description.replace("http://", "https://");
                http_warn = true;
            }
        }
        if license_description.contains("scripts.sil.org/OFL") {
            problems.push(Status::warn(
                "old-url",
                "Please consider updating the url from \
                 'https://scripts.sil.org/OFL' to \
                 'https://openfontlicense.org'.",
            ));
            continue;
        }
        if license_description != placeholder {
            problems.push(Status::fail(
                "wrong",
                &format!(
                    "License file {} exists but \
                          NameID 13 (LICENSE DESCRIPTION) value \
                          on platform {} is not specified for that.\
                          Value was: \"{}\"\n\
                           Must be changed to \"{}\"",
                    license_filename,
                    record.platform_id(),
                    license_description,
                    placeholder
                ),
            ));
        }

        if http_warn {
            problems.push(Status::warn(
                "http",
                "For now we're still accepting http URLs, \
                 but you should consider using https instead.",
            ));
        }
    }
    if !entry_found {
        problems.push(Status::fail(
            "missing",
            "Font lacks NameID 13 (LICENSE DESCRIPTION). \
             A proper licensing entry must be set.",
        ));
    }
    return_result(problems)
}
