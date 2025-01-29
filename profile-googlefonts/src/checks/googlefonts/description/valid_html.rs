use fontspector_checkapi::prelude::*;
use scraper::{Html, Selector};

#[check(
    id = "googlefonts/description/valid_html",
    rationale = "
        
        Sometimes people write malformed HTML markup. This check should ensure the
        file is good.

        Additionally, when packaging families for being pushed to the `google/fonts`
        git repo, if there is no DESCRIPTION.en_us.html file, some older versions of
        the `add_font.py` tool insert a placeholder description file which contains
        invalid html. This file needs to either be replaced with an existing
        description file or edited by hand.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/2664 and https://github.com/fonttools/fontbakery/issues/4829",
    title = "Is this a proper HTML snippet?",
    applies_to = "DESC"
)]
fn valid_html(desc: &Testable, _context: &Context) -> CheckFnResult {
    let mut problems = vec![];
    let content = std::str::from_utf8(&desc.contents)?;
    if content.contains("<html>") || content.contains("</html>") {
        problems.push(Status::fail(
            "html-tag",
            "DESCRIPTION file should not have an <html> tag, since it should only be a snippet that will later be included in the Google Fonts font family specimen webpage.",
        ));
    }
    let fragment = Html::parse_fragment(content);
    if !fragment.errors.is_empty() {
        problems.push(Status::fail(
            "malformed-snippet",
            &format!(
                "{} does not look like a proper HTML snippet. Please look for syntax errors. Maybe the following parser error message can help you find what's wrong:\n----------------\n{}\n----------------\n",
                desc.filename.as_os_str().to_string_lossy(),
                fragment.errors.join("\n")
            ),
        ));
    }
    #[allow(clippy::unwrap_used)] // it's a constant
    let selector = Selector::parse("p").unwrap();
    if fragment.select(&selector).count() == 0 {
        problems.push(Status::fail(
            "lacks-paragraph",
            &format!(
                "{} does not include an HTML <p> tag.",
                desc.filename.as_os_str().to_string_lossy()
            ),
        ));
    }
    return_result(problems)
}
