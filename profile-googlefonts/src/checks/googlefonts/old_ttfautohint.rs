use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use skrifa::string::StringId;

use crate::constants::{LATEST_TTFAUTOHINT_VERSION, TTFAUTOHINT_RE};

#[check(
    id = "googlefonts/old_ttfautohint",
    rationale = "
        Check if font has been hinted with an outdated version of ttfautohint.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Font has old ttfautohint applied?"
)]
fn old_ttfautohint(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    let ttfa_version = f
        .get_name_entry_strings(StringId::VERSION_STRING)
        .filter_map(|vstring| {
            TTFAUTOHINT_RE.captures(&vstring).map(|caps| {
                #[allow(clippy::unwrap_used)] // If there's some captures, there's two of them.
                caps.get(1).unwrap().as_str().to_string()
            })
        })
        .next();
    if let Some(ttfa_version) = ttfa_version {
        if let Ok(ttfa_version) = ttfa_version.parse::<semver::Version>() {
            #[allow(clippy::unwrap_used)] // It's a constant
            if ttfa_version < semver::Version::parse(LATEST_TTFAUTOHINT_VERSION).unwrap() {
                problems.push(Status::warn(
                    "old-ttfa",
                    &format!(
                        "ttfautohint used in font = {}; latest = {}; Need to re-run with the newer version!",
                        ttfa_version,
                        LATEST_TTFAUTOHINT_VERSION
                    ),
                ));
            }
        } else {
            problems.push(Status::fail(
                "parse-error",
                &format!(
                    "Failed to parse ttfautohint version values: latest = '{}'; used_in_font = '{}'",
                    LATEST_TTFAUTOHINT_VERSION,
                    ttfa_version
                ),
            ));
        }
    } else {
        problems.push(Status::info(
            "version-not-detected",
            &format!(
                "Could not detect which version of ttfautohint was used in this font. It is typically specified as a comment in the font version entries of the 'name' table. Such font version strings are currently: {}",
                f.get_name_entry_strings(StringId::VERSION_STRING).collect::<Vec<_>>().join(", ")
            ),
        ));
    }
    return_result(problems)
}
