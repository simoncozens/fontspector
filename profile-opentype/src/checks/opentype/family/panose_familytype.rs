use fontspector_checkapi::{prelude::*, FileTypeConvert, StatusCode, TestFont};
use read_fonts::TableProvider;

#[check(
    id = "opentype/family/panose_familytype",
    title = "Fonts have consistent PANOSE family type?",
    rationale = "
        The [PANOSE value](https://monotype.github.io/panose/) in the OS/2 table is a
        way of classifying a font based on its visual appearance and characteristics.

        The first field in the PANOSE classification is the family type: 2 means Latin
        Text, 3 means Latin Script, 4 means Latin Decorative, 5 means Latin Symbol.
        This check ensures that within a family, all fonts have the same family type.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",  // legacy check
    implementation = "all"
)]
fn panose_familytype(c: &TestableCollection, _context: &Context) -> CheckFnResult {
    let fonts = TTF.from_collection(c);
    let mut problems = vec![];
    let (ok, missing): (Vec<&TestFont>, Vec<&TestFont>) =
        fonts.iter().partition(|f| f.font().os2().is_ok());
    for font in missing {
        problems.push(Status::error(
            None,
            &format!(
                "Font {} is missing an OS/2 table",
                font.filename.to_string_lossy()
            ),
        ));
    }
    if !problems.is_empty() {
        return return_result(problems);
    }
    let panose_values = ok
        .iter()
        .map(|f| {
            #[allow(clippy::unwrap_used)]
            let panose_first = f.font().os2().unwrap().panose_10()[0];
            let panose_name = match panose_first {
                2 => "Latin Text".to_string(),
                3 => "Latin Script".to_string(),
                4 => "Latin Decorative".to_string(),
                5 => "Latin Symbol".to_string(),
                _ => format!("Unknown ({})", panose_first),
            };

            #[allow(clippy::unwrap_used)]
            (
                panose_first,
                panose_name,
                f.filename.file_name().unwrap().to_string_lossy(),
            )
        })
        .collect::<Vec<_>>();
    assert_all_the_same(
        _context,
        &panose_values,
        "inconsistency",
        "PANOSE family type is not the same across this family. In order to fix this, please make sure that the panose.bFamilyType value is the same in the OS/2 table of all of this family's font files.",
        StatusCode::Warn
    )
}
