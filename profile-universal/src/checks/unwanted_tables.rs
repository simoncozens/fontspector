use fontspector_checkapi::{fixfont, prelude::*, testfont, FileTypeConvert};
use read_fonts::types::Tag;
use write_fonts::FontBuilder;

const UNWANTED_TABLES: [(Tag, &str); 16] = [
    (Tag::new(b"DSIG"), "This font has a digital signature (DSIG table) which is only required - even if only a placeholder - on old programs like MS Office 2013 in order to work properly.\n
The current recommendation is to completely remove the DSIG table."),
    (Tag::new(b"FFTM"), "Table contains redundant FontForge timestamp info"),
    (Tag::new(b"TTFA"), "Redundant TTFAutohint table"),
    (Tag::new(b"TSI0"), "Table contains data only used in VTT"),
    (Tag::new(b"TSI1"), "Table contains data only used in VTT"),
    (Tag::new(b"TSI2"), "Table contains data only used in VTT"),
    (Tag::new(b"TSI3"), "Table contains data only used in VTT"),
    (Tag::new(b"TSI5"), "Table contains data only used in VTT"),
    (Tag::new(b"TSIC"), "Table contains data only used in VTT"),
    (Tag::new(b"TSIV"), "Table contains data only used in VOLT"),
    (Tag::new(b"TSIP"), "Table contains data only used in VOLT"),
    (Tag::new(b"TSIS"), "Table contains data only used in VOLT"),
    (Tag::new(b"TSID"), "Table contains data only used in VOLT"),
    (Tag::new(b"TSIJ"), "Table contains data only used in VOLT"),
    (Tag::new(b"TSIB"), "Table contains data only used in VOLT"),
    (Tag::new(b"prop"), "Table used on AAT, Apple's OS X specific technology. Although Harfbuzz now has optional AAT support, new fonts should not be using that.")
];

#[check(
    id = "unwanted_tables",
    title = "Are there unwanted tables?",
    rationale = "Some font editors store source data in their own SFNT tables, and these can sometimes sneak into final release files, which should only have OpenType spec tables.",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",  // legacy check
    hotfix = delete_unwanted_tables
)]
fn unwanted_tables(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);

    let mut reasons = vec![];
    for (table, reason) in UNWANTED_TABLES.iter() {
        if f.font().table_data(*table).is_some() {
            reasons.push(format!("Table: `{}` Reason: {}\n", table, reason));
        }
    }
    Ok(if !reasons.is_empty() {
        Status::just_one_fail(
            "unwanted-tables",
            &format!("Unwanted tables found:\n {}", reasons.join("\n")),
        )
    } else {
        Status::just_one_pass()
    })
}

fn delete_unwanted_tables(t: &Testable) -> FixFnResult {
    let f = fixfont!(t);
    let unwanted_tags = UNWANTED_TABLES
        .iter()
        .map(|(tag, _)| tag)
        .collect::<Vec<_>>();
    let mut new_font = FontBuilder::new();
    for table in f.font().table_directory.table_records() {
        let tag = table.tag.get();
        if !unwanted_tags.contains(&&tag) {
            if let Some(table) = f.font().table_data(tag) {
                new_font.add_raw(tag, table);
            }
        }
    }
    let new_bytes = new_font.build();
    std::fs::write(&t.filename, new_bytes).map_err(|_| "Couldn't write file".to_string())?;
    Ok(true)
}
