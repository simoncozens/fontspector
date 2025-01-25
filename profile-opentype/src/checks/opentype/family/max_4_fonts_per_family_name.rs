use std::collections::HashMap;

use fontspector_checkapi::{prelude::*, FileTypeConvert};
use read_fonts::types::NameId;


#[check(
    id = "opentype/family/max_4_fonts_per_family_name",
    rationale = "
        Per the OpenType spec:

        'The Font Family name [...] should be shared among at most four fonts that
        differ only in weight or style [...]'
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/2372",
    title = "Verify that each group of fonts with the same nameID 1 has maximum of 4 fonts.",
    implementation = "all"
)]
fn max_4_fonts_per_family_name(t: &TestableCollection, _context: &Context) -> CheckFnResult {
    let fonts = TTF.from_collection(t);
    let mut counter = HashMap::new();
    let mut problems = vec![];
    for font in fonts {
        let family_name = font
            .get_name_entry_strings(NameId::FAMILY_NAME)
            .next()
            .ok_or_else(|| {
                CheckError::Error(format!(
                    "Font {} is missing a Family Name entry",
                    font.filename.to_string_lossy()
                ))
            })?;
        *counter.entry(family_name).or_insert(0) += 1;
    }
    for (family_name, count) in counter {
        if count > 4 {
            problems.push(Status::fail(
                "too-many",
                &format!(
                    "Family name '{}' has {} fonts, which is more than the maximum of 4",
                    family_name, count
                ),
            ));
        }
    }

    return_result(problems)
}
