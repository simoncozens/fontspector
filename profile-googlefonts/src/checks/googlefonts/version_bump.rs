use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use read_fonts::TableProvider;

use crate::network_conditions::remote_styles;

#[check(
    id = "googlefonts/version_bump",
    rationale = "
        
        We check that the version number has been bumped since the last release on
        Google Fonts. This helps to ensure that the version being PRed is newer than
        the one currently hosted on fonts.google.com.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Version number has increased since previous release on Google Fonts?"
)]
fn version_bump(f: &Testable, context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    skip!(
        context.skip_network,
        "network-check",
        "Skipping network check"
    );
    let family_name = font.best_familyname().ok_or(CheckError::Error(
        "Could not determine family name".to_string(),
    ))?;
    let remote_fonts = remote_styles(&family_name, context).map_err(CheckError::Error)?;
    let a_remote_font = remote_fonts.first().ok_or(CheckError::Error(format!(
        "Couldn't get remote font for {}",
        family_name
    )))?;
    let a_remote_font = testfont!(a_remote_font);

    let local_version = font.font().head()?.font_revision();
    let remote_version = a_remote_font.font().head()?.font_revision();
    if local_version == remote_version {
        Ok(Status::just_one_fail(
            "same-version",
            &format!(
                "Version number {} is equal to version on Google fonts",
                local_version.to_f32()
            ),
        ))
    } else {
        Ok(Status::just_one_pass())
    }
}
