use font_types::Tag;
use fontspector_checkapi::{prelude::*, FileTypeConvert};
use write_fonts::FontBuilder;

const UNWANTED_TABLES: [(Tag, &str); 8] = [
    (Tag::new(b"FFTM"), "Table contains redundant FontForge timestamp info"),
    (Tag::new(b"TTFA"), "Redundant TTFAutohint table"),
    (Tag::new(b"TSI0"), "Table contains data only used in VTT"),
    (Tag::new(b"TSI1"), "Table contains data only used in VTT"),
    (Tag::new(b"TSI2"), "Table contains data only used in VTT"),
    (Tag::new(b"TSI3"), "Table contains data only used in VTT"),
    (Tag::new(b"TSI5"), "Table contains data only used in VTT"),
    (Tag::new(b"prop"), "Table used on AAT, Apple's OS X specific technology. Although Harfbuzz now has optional AAT support, new fonts should not be using that.")
];

fn unwanted_tables(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = TTF.from_testable(t).ok_or("Not a TTF file")?;

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
    let f = TTF.from_testable(t).ok_or("Not a TTF file".to_string())?;
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

pub const CHECK_UNWANTED_TABLES: Check = Check {
    id: "com.google.fonts/check/unwanted_tables",
    title: "Are there unwanted tables?",
    rationale: "Some font editors store source data in their own SFNT tables, and these can sometimes sneak into final release files, which should only have OpenType spec tables.",
    proposal: "legacy:check/053",
    check_one: Some(&unwanted_tables),
    check_all: None,
    applies_to: "TTF",
    hotfix: Some(&delete_unwanted_tables),
    fix_source: None,
    flags: CheckFlags::default(),
};
