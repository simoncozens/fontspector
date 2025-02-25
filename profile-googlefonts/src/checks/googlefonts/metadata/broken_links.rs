use crate::{checks::googlefonts::metadata::family_proto, network_conditions::get_url};
use fontspector_checkapi::{prelude::*, skip};
use reqwest::StatusCode;

#[check(
    id = "googlefonts/metadata/broken_links",
    rationale = "
        
        This check ensures that any URLs found within the copyright
        field of the METADATA.pb file are valid.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/2550 and https://github.com/fonttools/fontbakery/issues/4110",
    title = "Does METADATA.pb copyright field contain broken links?",
    applies_to = "MDPB"
)]
fn broken_links(c: &Testable, context: &Context) -> CheckFnResult {
    let mut problems = vec![];
    skip!(
        context.skip_network,
        "network-check",
        "Skipping network check"
    );
    let msg = family_proto(c).map_err(|e| {
        CheckError::Error(format!("METADATA.pb is not a valid FamilyProto: {:?}", e))
    })?;
    let mut unique_links: Vec<String> = vec![];
    let mut broken = vec![];
    for font_metadata in msg.fonts {
        let copyright = font_metadata.copyright();
        if copyright.contains("mailto:") {
            if unique_links.contains(&copyright.to_string()) {
                continue;
            }
            unique_links.push(copyright.to_string());
            problems.push(Status::fail(
                "email",
                &format!("Found an email address: {}", &copyright),
            ));
            continue;
        }
        if !copyright.contains("http") {
            continue;
        }
        #[allow(clippy::unwrap_used)] // we just verified it's there
        let mut link = format!("http{}", &copyright.split("http").nth(1).unwrap());
        for endchar in [" ", ")"] {
            if let Some(split) = link.split(endchar).next() {
                link = split.to_string();
            }
        }
        if unique_links.contains(&link.to_string()) {
            continue;
        }
        unique_links.push(link.to_string());
        if let Err(error) = get_url(context, &link) {
            if error.is_timeout() {
                problems.push(Status::warn("timeout", &format!("Timedout while attempting to access: '{}'. Please verify if that's a broken link.", link)));
            } else if error.status() == Some(StatusCode::TOO_MANY_REQUESTS) {
                // Probably OK
            } else if let Some(status) = error.status() {
                broken.push(format!("{} (status code: {})", link, status));
            } else {
                broken.push(format!("{} (error: {})", link, error));
            }
        }
    }

    // Additional (non-fontbakery) check: let's make sure the sources.repository_url is valid:
    let repo_url = msg.source.repository_url();
    if !repo_url.is_empty() && !unique_links.contains(&repo_url.to_string()) {
        if let Err(error) = get_url(context, repo_url) {
            if error.is_timeout() {
                problems.push(Status::warn("timeout", &format!("Timedout while attempting to access: '{}'. Please verify if that's a broken link.", repo_url)));
            } else if error.status() == Some(StatusCode::TOO_MANY_REQUESTS) {
                // Probably OK
            } else {
                problems.push(Status::fail(
                    "broken-repo-url",
                    &format!("The repository url {} is broken: {}", repo_url, error),
                ));
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
