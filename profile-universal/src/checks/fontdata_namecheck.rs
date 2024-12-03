use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use read_fonts::tables::name::NameId;
use skrifa::MetadataProvider;

const NAMECHECK_URL: &str = "http://namecheck.fontdata.com";

#[check(
    id = "fontdata_namecheck",
    rationale = "
        We need to check names are not already used, and today the best place to check
        that is http://namecheck.fontdata.com
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/494",
    title = "Familyname must be unique according to namecheck.fontdata.com"
)]
fn fontdata_namecheck(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(
        context.skip_network,
        "network-check",
        "Skipping network check"
    );
    let name = f
        .font()
        .localized_strings(NameId::FAMILY_NAME)
        .english_or_first()
        .ok_or(CheckError::Error("Family name not found".to_string()))?
        .to_string();
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_10_1)")
        .timeout(context.network_timeout.map(std::time::Duration::from_secs))
        .build()?;
    let response = client
        .post(NAMECHECK_URL)
        .query(&[("q", name.clone())])
        .send()
        .map_err(|e| CheckError::Error(format!("Failed to access: {}. {}", NAMECHECK_URL, e)))?;
    let data = response
        .text()
        .map_err(|e| CheckError::Error(format!("Failed to decode response: {}", e)))?;
    Ok(if data.contains("fonts by that exact name") {
        Status::just_one_info(
            "name-collision",
            &format!(
                r#"The family name "{}" seems to be already in use.
Please visit {} for more info."#,
                name, NAMECHECK_URL
            ),
        )
    } else {
        Status::just_one_pass()
    })
}
