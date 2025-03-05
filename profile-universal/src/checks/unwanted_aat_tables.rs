use fontspector_checkapi::{fixfont, prelude::*, testfont, FileTypeConvert};
use write_fonts::FontBuilder;

const UNWANTED_TABLES: [&[u8; 4]; 23] = [
    b"acnt", b"ankr", b"bdat", b"bhed", b"bloc", b"bmap", b"bsln", b"EBSC", b"fdsc", b"feat",
    b"fond", b"gcid", b"just", b"kerx", b"lcar", b"ltag", b"mort", b"morx", b"opbd", b"prop",
    b"trak", b"xref", b"Zaph",
];

#[check(
    id = "unwanted_aat_tables",
    title = "Are there unwanted Apple tables?",
    rationale = "
        Apple's TrueType reference manual [1] describes SFNT tables not in the
        Microsoft OpenType specification [2] and these can sometimes sneak into final
        release files.

        This check ensures fonts only have OpenType tables.

        [1] https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6.html
        [2] https://docs.microsoft.com/en-us/typography/opentype/spec/
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/2190",
    hotfix = delete_unwanted_aat_tables
)]
fn unwanted_aat_tables(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);

    let mut found = vec![];
    for tag in UNWANTED_TABLES.iter() {
        if f.has_table(tag) {
            found.push(String::from_utf8(tag.to_vec()).map_err(|_| {
                CheckError::Error(format!("Font tag '{:?}' wasn't UTF8?", tag.to_vec()))
            })?);
        }
    }
    Ok(if !found.is_empty() {
        Status::just_one_fail(
            "has-unwanted-tables",
            &format!(
                "Unwanted AAT tables were found in the font and should be removed,
                       either by fonttools/ttx or by editing them using the tool
                       they're built with:\n\n{}",
                found.join("\n")
            ),
        )
    } else {
        Status::just_one_pass()
    })
}

fn delete_unwanted_aat_tables(t: &mut Testable) -> FixFnResult {
    let f = fixfont!(t);
    let mut new_font = FontBuilder::new();
    for table in f.font().table_directory.table_records() {
        let tag = table.tag.get();
        if !UNWANTED_TABLES.contains(&&tag.into_bytes()) {
            if let Some(table) = f.font().table_data(tag) {
                new_font.add_raw(tag, table);
            }
        }
    }
    let new_bytes = new_font.build();
    t.set(new_bytes);
    Ok(true)
}
