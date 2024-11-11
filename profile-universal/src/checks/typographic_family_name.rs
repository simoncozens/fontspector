use fontspector_checkapi::{prelude::*, FileTypeConvert, StatusCode};
use read_fonts::types::NameId;

#[check(
    id = "typographic_family_name",
    rationale = "
        Check whether Name ID 16 (Typographic Family name) is consistent
        across the set of fonts.
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/4567",
    title = "Typographic Family name consistency.",
    implementation = "all"
)]
fn typographic_family_name(c: &TestableCollection, context: &Context) -> CheckFnResult {
    let ttfs = TTF.from_collection(c);
    let items: Vec<_> = ttfs
        .iter()
        .map(|f| {
            let name = f
                .get_name_entry_strings(NameId::TYPOGRAPHIC_FAMILY_NAME)
                .next()
                .unwrap_or("<missing>".to_string());
            #[allow(clippy::unwrap_used)]
            (
                name.clone(),
                name,
                f.filename.file_name().unwrap().to_string_lossy(),
            )
        })
        .collect();
    assert_all_the_same(
        context,
        &items,
        "inconsistency",
        "Name ID 16 (Typographic Family name) is not consistent across fonts.",
        StatusCode::Fail,
    )
}
