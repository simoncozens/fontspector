use fontspector_checkapi::prelude::*;
use scraper::{Html, Selector};

const UNSUPPORTED_ELEMENTS: [&str; 16] = [
    "applet", "base", "embed", "form", "frame", "frameset", "head", "iframe", "link", "math",
    "meta", "object", "script", "style", "svg", "template",
];

#[check(
    id = "googlefonts/description/has_unsupported_elements",
    rationale = "
        
        The Google Fonts backend doesn't support the following html elements:
        https://googlefonts.github.io/gf-guide/description.html#requirements
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/2811#issuecomment-1907566857",
    title = "Check the description doesn't contain unsupported html elements",
    applies_to = "DESC"
)]
fn has_unsupported_elements(desc: &Testable, _context: &Context) -> CheckFnResult {
    let mut problems = vec![];
    let fragment = Html::parse_fragment(std::str::from_utf8(&desc.contents)?);
    #[allow(clippy::unwrap_used)] // it's a constant
    let selector = Selector::parse(&UNSUPPORTED_ELEMENTS.join(",")).unwrap();
    let unsupported = fragment
        .select(&selector)
        .map(|x| x.value().name())
        .collect::<Vec<_>>();
    if !unsupported.is_empty() {
        problems.push(Status::error(
            Some("unsupported-elements"),
            &format!(
                "The DESCRIPTION file contains unsupported html element(s). Please remove: {}",
                unsupported.join(", ")
            ),
        ));
    }
    #[allow(clippy::unwrap_used)] // it's a constant
    let bad_video = Selector::parse("video:not([src])").unwrap();
    if fragment.select(&bad_video).count() > 0 {
        problems.push(Status::error(
            Some("video-tag-needs-src"),
            &format!(
                "{} contains a video tag with no src attribute.",
                desc.filename.as_os_str().to_string_lossy()
            ),
        ));
    }
    return_result(problems)
}
