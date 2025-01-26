use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use itertools::Itertools;
use skrifa::MetadataProvider;

use google_fonts_axisregistry::AxisRegistry;

#[check(
    id = "googlefonts/axisregistry/fvar_axis_defaults",
    rationale = "
        
        Check that axis defaults have a corresponding fallback name registered at the
        Google Fonts Axis Registry, available at
        https://github.com/google/fonts/tree/main/axisregistry

        This is necessary for the following reasons:

        To get ZIP files downloads on Google Fonts to be accurate — otherwise the
        chosen default font is not generated. The Newsreader family, for instance, has
        a default value on the 'opsz' axis of 16pt. If 16pt was not a registered
        fallback position, then the ZIP file would instead include another position
        as default (such as 14pt).

        For the Variable fonts to display the correct location on the specimen page.

        For VF with no weight axis to be displayed at all. For instance, Ballet, which
        has no weight axis, was not appearing in sandbox because default position on
        'opsz' axis was 16pt, and it was not yet a registered fallback positon.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3141",
    title = "
    Validate defaults on fvar table match registered fallback names in GFAxisRegistry.
    "
)]
fn fvar_axis_defaults(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    skip!(
        !f.is_variable_font(),
        "not-variable",
        "Font is not variable"
    );
    let registry = AxisRegistry::new();
    for axis in f.font().axes().iter() {
        let tag = axis.tag().to_string();
        if let Some(entry) = registry.get(&tag) {
            let fallbacks = entry.fallback.iter().map(|f| f.value()).collect::<Vec<_>>();
            if !fallbacks.contains(&axis.default_value()) {
                problems.push(Status::fail(
                "not-registered",
                &format!(
                    "The defaul value {}:{} is not registered as an axis fallback name on the Google Axis Registry.\n\tYou should consider suggesting the addition of this value to the registry or adopted one of the existing fallback names for this axis:\n\t{}",
                    tag,
                    axis.default_value(),
                    fallbacks.iter().map(|x| x.to_string()).join(", ")
                ),
            ));
            }
        }
    }

    return_result(problems)
}
