use std::time::Duration;

use fontspector_checkapi::{prelude::*, skip};
use hashbrown::HashSet;
use scraper::{Html, Selector};

#[check(
    id = "googlefonts/description/broken_links",
    rationale = "
        
        The snippet of HTML in the DESCRIPTION.en_us.html/ARTICLE.en_us.html file is
        added to the font family webpage on the Google Fonts website. For that reason,
        all hyperlinks in it must be properly working.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4110 and https://github.com/fonttools/fontbakery/issues/4829",
    title = "Does DESCRIPTION file contain broken links?",
    applies_to = "DESC"
)]
fn broken_links(desc: &Testable, context: &Context) -> CheckFnResult {
    let mut problems = vec![];
    skip!(
        context.skip_network,
        "network-check",
        "Skipping network check"
    );
    let fragment = Html::parse_fragment(std::str::from_utf8(&desc.contents)?);
    #[allow(clippy::unwrap_used)] // it's a constant
    let selector = Selector::parse("a[href]").unwrap();
    let mut done = HashSet::new();
    let mut broken = HashSet::new();
    for element in fragment.select(&selector) {
        #[allow(clippy::unwrap_used)] // we know there's a href
        let href = element.value().attr("href").unwrap();
        if done.contains(href) {
            continue;
        }
        done.insert(href);
        if href.starts_with("mailto:") {
            #[allow(clippy::unwrap_used)] // we know there's a @
            if href.contains('@') && href.split('@').nth(1).unwrap().contains('.') {
                problems.push(Status::fail(
                    "email",
                    &format!("Found an email address: {}", href),
                ));
            }
            continue;
        }
        let mut request = reqwest::blocking::Client::new().head(href);
        if let Some(timeout) = context.network_timeout {
            request = request.timeout(Duration::new(timeout, 0));
        }
        match request.send() {
            Ok(response) => {
                if !response.status().is_success() {
                    broken.insert(format!("{} (status code: {})", href, response.status()));
                }
            }
            Err(error) => {
                if error.is_timeout() {
                    problems.push(Status::warn("timeout", &format!("Timedout while attempting to access: '{}'. Please verify if that's a broken link.", href)));
                } else {
                    broken.insert(format!("{} (error: {})", href, error));
                }
            }
        }
    }
    if !broken.is_empty() {
        problems.push(Status::fail(
            "broken-links",
            &format!(
                "The following links are broken:\n{}",
                bullet_list(context, broken)
            ),
        ));
    }

    return_result(problems)
}
