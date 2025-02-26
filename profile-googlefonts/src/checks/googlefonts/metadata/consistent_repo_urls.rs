use fontspector_checkapi::prelude::*;

use crate::checks::googlefonts::metadata::family_proto;

fn clean_url(url: &str) -> String {
    let mut cleaned = url.trim().to_string();
    if let Some(split) = cleaned.split(")").next() {
        cleaned = split.to_string();
    }
    if cleaned.ends_with('/') {
        cleaned.pop();
    }
    if cleaned.ends_with(".git") {
        let _ = cleaned.split_off(cleaned.len() - 4);
    }
    cleaned
}

#[check(
    id = "googlefonts/metadata/consistent_repo_urls",
    rationale = "
        
        Sometimes, perhaps due to copy-pasting, projects may declare different URLs
        between the font.coyright and the family.sources.repository_url fields.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4056",
    title = "METADATA.pb: Check URL on copyright string is the same as in repository_url field.",
    implementation = "all"
)]
fn consistent_repo_urls(c: &TestableCollection, context: &Context) -> CheckFnResult {
    let mdpb = c
        .get_file("METADATA.pb")
        .ok_or_else(|| CheckError::skip("no-mdpb", "No METADATA.pb file found"))?;
    let msg = family_proto(mdpb)?;
    let repo_url = clean_url(msg.source.repository_url());
    if repo_url.is_empty() {
        return Ok(Status::just_one_fail(
            "lacks-repo-url",
            "Please add a family.source.repository_url entry.",
        ));
    }

    let mut bad_urls = vec![];

    for font in msg.fonts {
        if let Some(httpbit) = font.copyright().split("http").nth(1) {
            let link = clean_url(&format!("http{}", httpbit));
            if link != repo_url {
                bad_urls.push(("font copyright string", link));
            }
        }
    }

    if let Some(ofl) = c.get_file("OFL.txt") {
        let license_contents = String::from_utf8(ofl.contents.clone())
            .map_err(|e| CheckError::Error(format!("OFL.txt is not valid UTF-8: {:?}", e)))?;
        let first_line = license_contents.lines().next().unwrap_or_default();
        if first_line.contains("http") {
            let link = clean_url(first_line.split("http").nth(1).unwrap_or_default());
            if link != repo_url {
                bad_urls.push(("OFL text", link));
            }
        }
    }

    if let Some(description) = c.get_file("DESCRIPTION.en_us.html") {
        let description_contents =
            String::from_utf8(description.contents.clone()).map_err(|e| {
                CheckError::Error(format!(
                    "DESCRIPTION.en_us.html is not valid UTF-8: {:?}",
                    e
                ))
            })?;
        let headless = repo_url
            .trim_start_matches("https://")
            .trim_start_matches("http://");
        for match_ in description_contents.split_whitespace() {
            if match_.contains("github.com/") {
                let link = clean_url(match_);
                if link != headless {
                    bad_urls.push(("HTML description", link));
                }
            }
        }
    }

    if !bad_urls.is_empty() {
        return Ok(Status::just_one_fail(
            "mismatch",
            &format!(
                "Repository URL is {}. But:\n{}",
                repo_url,
                bullet_list(
                    context,
                    bad_urls
                        .iter()
                        .map(|(location, url)| format!("{} has '{}'", location, url))
                )
            ),
        ));
    }
    return Ok(Status::just_one_pass());
}
