use std::sync::LazyLock;

use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use hashbrown::HashSet;
use itertools::Itertools;

const CAMELCASE_EXCEPTIONS_FILE: &str =
    include_str!("../../../../resources/camelcased_familyname_exceptions.txt");
static CAMELCASE_EXCEPTIONS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    CAMELCASE_EXCEPTIONS_FILE
        .lines()
        .flat_map(|line| line.split('#').next())
        .map(|x| x.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
});
const ABBREVIATION_EXCEPTIONS_FILE: &str =
    include_str!("../../../../resources/abbreviations_familyname_exceptions.txt");
static ABBREVIATION_EXCEPTIONS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    ABBREVIATION_EXCEPTIONS_FILE
        .lines()
        .flat_map(|line| line.split('#').next())
        .map(|x| x.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
});

#[check(
    id = "googlefonts/name/family_name_compliance",
    rationale = "
        
        Checks the family name for compliance with the Google Fonts Guide.
        https://googlefonts.github.io/gf-guide/onboarding.html#new-fonts

        If you want to have your family name added to the CamelCase
        exceptions list, please submit a pull request to the
        camelcased_familyname_exceptions.txt file.

        Similarly, abbreviations can be submitted to the
        abbreviations_familyname_exceptions.txt file.

        These are located in the Lib/fontbakery/data/googlefonts/ directory
        of the FontBakery source code currently hosted at
        https://github.com/fonttools/fontbakery/
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4049",
    title = "Check family name for GF Guide compliance."
)]
fn family_name_compliance(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let family_name = f
        .best_familyname()
        .ok_or(CheckError::Error("Couldn't determine family name".into()))?;
    let family_name = family_name.strip_suffix(" SC").unwrap_or(&family_name);
    let mut problems = vec![];
    if family_name
        .strip_suffix(" SC")
        .unwrap_or(family_name)
        .chars()
        .tuple_windows()
        .any(|(a, b)| a.is_ascii_lowercase() && b.is_ascii_uppercase())
        && !CAMELCASE_EXCEPTIONS.contains(family_name)
    {
        problems.push(Status::fail(
            "camelcase",
            &format!(
                "\"{}\" is a CamelCased name. To solve this, simply use spaces instead in the font name.",
                family_name
            ),
        ));
    }
    if family_name
        .chars()
        .tuple_windows()
        .any(|(a, b)| a.is_ascii_uppercase() && b.is_ascii_uppercase())
        && !ABBREVIATION_EXCEPTIONS.iter().any(|exception| {
            family_name.contains(exception) // This is very slack, but it's what the original code does.
        })
    {
        problems.push(Status::fail(
            "abbreviation",
            &format!("\"{}\" contains an abbreviation.", family_name),
        ));
    }

    let forbidden_characters = family_name
        .chars()
        .filter(|&c| !c.is_ascii_alphanumeric() && c != ' ')
        .sorted()
        .dedup()
        .collect::<String>();

    if !forbidden_characters.is_empty() {
        problems.push(Status::fail(
            "forbidden-characters",
            &format!(
                "\"{}\" contains the following characters which are not allowed: \"{}\".",
                family_name, forbidden_characters
            ),
        ));
    }

    if !family_name.starts_with(|c: char| c.is_ascii_uppercase()) {
        problems.push(Status::fail(
            "starts-with-not-uppercase",
            &format!(
                "\"{}\" doesn't start with an uppercase letter.",
                family_name
            ),
        ));
    }
    return_result(problems)
}
