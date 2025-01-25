use fontspector_checkapi::{prelude::*, FileTypeConvert, StatusCode};
use read_fonts::{ReadError, TableProvider};


#[check(
    id = "opentype/family/equal_font_versions",
    title = "Make sure all font files have the same version value.",
    rationale = "Within a family released at the same time, all members of the family should have the same version number in the head table.",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",  // legacy check
    implementation = "all"
)]
fn equal_font_versions(c: &TestableCollection, context: &Context) -> CheckFnResult {
    let fonts = TTF.from_collection(c);
    let versions_names: Result<Vec<_>, ReadError> = fonts
        .iter()
        .map(|f| {
            f.font().head().map(|head| {
                (
                    head.font_revision(),
                    format!("{:.03}", head.font_revision().to_f32()),
                    f.filename.to_string_lossy(),
                )
            })
        })
        .collect();
    assert_all_the_same(
        context,
        &versions_names?,
        "mismatch",
        "Version info differs among font files of the same font project.",
        StatusCode::Warn,
    )
}
