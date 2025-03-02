use crate::checks::googlefonts::metadata::family_proto;
use fontspector_checkapi::prelude::*;

#[check(
    id = "googlefonts/description/has_article",
    rationale = "
        
        Fonts may have a longer article about them, or a description, but
        not both - except for Noto fonts which should have both!
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3841",
    proposal = "https://github.com/fonttools/fontbakery/issues/4318",
    proposal = "https://github.com/fonttools/fontbakery/issues/4702",
    title = "Check for presence of an ARTICLE.en_us.html file",
    implementation = "all"
)]
fn has_article(c: &TestableCollection, _context: &Context) -> CheckFnResult {
    let mut problems = vec![];
    let article = c.get_file("ARTICLE.en_us.html");
    let description = c.get_file("DESCRIPTION.en_us.html");
    let article_is_empty = article.map(|t| t.contents.is_empty()).unwrap_or(false);
    let description_is_empty = description.map(|t| t.contents.is_empty()).unwrap_or(false);
    let is_noto = c
        .get_file("METADATA.pb")
        .and_then(|t| family_proto(t).ok())
        .map(|msg| msg.name().starts_with("Noto "))
        .unwrap_or(false);
    if !is_noto {
        if article.is_none() {
            problems.push(Status::fail(
                "missing-article",
                "This font doesn't have an ARTICLE.en_us.html file.",
            ));
        } else {
            if article_is_empty {
                problems.push(Status::fail(
                    "empty-article",
                    "The ARTICLE.en_us.html file is empty.",
                ));
            }
            if description.is_some() {
                problems.push(Status::fail(
                    "description-and-article",
                    "This font has both a DESCRIPTION.en_us.html file and an ARTICLE.en_us.html file. In this case the description must be deleted.",
                ));
            }
        }
    } else {
        if article.is_none() {
            problems.push(Status::fail(
                "missing-article",
                "This is a Noto font but it lacks an ARTICLE.en_us.html file.",
            ));
        }
        if article_is_empty {
            problems.push(Status::fail(
                "empty-article",
                "The ARTICLE.en_us.html file is empty.",
            ));
        }
        if description.is_none() || description_is_empty {
            problems.push(Status::fail(
                "missing-description",
                "This is a Noto font but it lacks a DESCRIPTION.en_us.html file.",
            ));
        }
    }
    return_result(problems)
}
