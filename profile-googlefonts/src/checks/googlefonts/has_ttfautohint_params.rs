use crate::constants::TTFAUTOHINT_RE;
use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use skrifa::string::StringId;

#[check(
    id = "googlefonts/has_ttfautohint_params",
    rationale = "
        
        It is critically important that all static TTFs in the Google Fonts API
        which were autohinted with ttfautohint store their TTFAutohint args in
        the 'name' table, so that an automated solution can be made to
        replicate the hinting on subsets, etc.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/1773",
    title = "Font has ttfautohint params?"
)]
fn has_ttfautohint_params(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    let mut passed = false;
    for vstring in f.get_name_entry_strings(StringId::VERSION_STRING) {
        if let Some(caps) = TTFAUTOHINT_RE.captures(&vstring) {
            #[allow(clippy::unwrap_used)] // If there's some captures, there's two of them.
            let params = caps.get(2).unwrap().as_str();
            if !params.is_empty() {
                problems.push(Status::info(
                    "ok",
                    &format!("Font has ttfautohint params ({})", params),
                ));
                passed = true;
            }
        } else {
            problems.push(Status::skip(
                "not-hinted",
                "Font appears to our heuristic as not hinted using ttfautohint.",
            ));
            passed = true;
        }
    }
    if !passed {
        problems.push(Status::fail(
            "lacks-ttfa-params",
            "Font is lacking ttfautohint params on its version strings on the name table.",
        ));
    }
    return_result(problems)
}
