use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use skrifa::string::StringId;

#[derive(Debug, PartialEq)]
struct License {
    filename: &'static str,
    url: &'static str,
    placeholder: &'static str,
}

const LEGACY_UFL_FAMILIES: [&str; 4] = ["Ubuntu", "Ubuntu Sans", "Ubuntu Mono", "Ubuntu Sans Mono"];
const KNOWN_LICENSES: [License; 3] =  [
    License {
        filename: "OFL.txt",
        url: "https://openfontlicense.org",
        placeholder: "This Font Software is licensed under the SIL Open Font License, Version 1.1. This license is available with a FAQ at: https://openfontlicense.org",
    },
    License {
        filename: "LICENSE.txt",
        url: "https://www.apache.org/licenses/LICENSE-2.0",
        placeholder: "Licensed under the Apache License, Version 2.0",
    },
    License {
        filename: "UFL.txt",
        url: "https://www.ubuntu.com/legal/terms-and-policies/font-licence",
        placeholder: "Licensed under the Ubuntu Font Licence 1.0.",
    },
];

fn identify_license(name_string: &str) -> Option<&'static License> {
    KNOWN_LICENSES
        .iter()
        .find(|&license| name_string.replace("http://", "https://") == license.placeholder)
}

#[check(
    id = "googlefonts/name/license_url",
    rationale = "
        
        A known license URL must be provided in the NameID 14 (LICENSE INFO URL)
        entry of the name table.

        The source of truth for this check is the licensing text found on the NameID 13
        entry (LICENSE DESCRIPTION).

        The string snippets used for detecting licensing terms are:

        - \"This Font Software is licensed under the SIL Open Font License, Version 1.1.
          This license is available with a FAQ at: openfontlicense.org\"

        - \"Licensed under the Apache License, Version 2.0\"

        - \"Licensed under the Ubuntu Font Licence 1.0.\"


        Currently accepted licenses are Apache or Open Font License. For a small set of
        legacy families the Ubuntu Font License may be acceptable as well.

        When in doubt, please choose OFL for new font projects.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4358",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "License URL matches License text on name table?"
)]
fn license_url(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    let mut detected_license = None;
    for license_desc in f.get_name_entry_strings(StringId::LICENSE_DESCRIPTION) {
        if license_desc.contains("http://") {
            problems.push(Status::warn(
                "http-in-description",
                "Please consider using HTTPS URLs in the license description",
            ));
        }
        detected_license = identify_license(&license_desc);
        if detected_license.is_some() {
            break;
        }
    }

    if detected_license == Some(&KNOWN_LICENSES[2])
        && !LEGACY_UFL_FAMILIES
            .iter()
            .any(|lf| f.best_familyname() == Some(lf.to_string()))
    {
        problems.push(Status::fail(
            "ufl",
            "The Ubuntu Font License is only acceptable on the Google Fonts collection for legacy font families that already adopted such license. New Families should use either Apache or Open Font License.",
        ));
        return return_result(problems);
    }

    if let Some(detected_license) = detected_license {
        let expected_url = detected_license.url;
        for found_url in f.get_name_entry_strings(StringId::LICENSE_URL) {
            if found_url.contains("http://") {
                problems.push(Status::warn(
                    "http-in-license-info",
                    "Please consider using HTTPS URLs in the license info URL",
                ));
            }
            if found_url.replace("http://", "https://") == expected_url {
                return return_result(problems);
            } else if found_url.contains("scripts.sil.org/OFL") {
                problems.push(Status::warn(
                    "deprecated-ofl-url",
                    "OFL url is no longer \"https://scripts.sil.org/OFL\". Use 'https://openfontlicense.org' instead.",
                ));
            } else {
                problems.push(Status::fail(
                    "licensing-inconsistency",
                    &format!(
                        "Licensing inconsistency in name table entries! NameID={:?} (LICENSE DESCRIPTION) indicates {} licensing, but NameID={} (LICENSE URL) has '{}'. Expected: '{}'",
                        StringId::LICENSE_DESCRIPTION,
                        detected_license.filename.replace(".txt", ""),
                        StringId::LICENSE_URL,
                        found_url,
                        expected_url,
                    ),
                ));
            }
        }
    } else {
        problems.push(Status::skip(
            "no-license-found",
            "Could not infer the font license. Please ensure NameID 13 (LICENSE DESCRIPTION) is properly set.",
        ));
        return return_result(problems);
    }

    return_result(problems)
}
