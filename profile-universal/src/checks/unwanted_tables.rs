use fontspector_checkapi::{prelude::*, FileTypeConvert};
use skrifa::Tag;

fn unwanted_tables(t: &Testable) -> StatusList {
    let f = TTF.from_testable(t).expect("Not a TTF file");
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
    let mut reasons = vec![];
    for (table, reason) in UNWANTED_TABLES.iter() {
        if f.font().table_data(*table).is_some() {
            reasons.push(format!("Table: {} Reason: {}", table, reason));
        }
    }
    if !reasons.is_empty() {
        Status::just_one_fail(&format!("Unwanted tables found: {}", reasons.join("\n")))
    } else {
        Status::just_one_pass()
    }
}

pub const UNWANTED_TABLES_CHECK: Check = Check {
    id: "com.google.fonts/check/unwanted_tables",
    title: "Are there unwanted tables?",
    rationale: Some("Some font editors store source data in their own SFNT tables, and these can sometimes sneak into final release files, which should only have OpenType spec tables."),
    proposal: Some("legacy:check/053"),
    check_one: Some(&unwanted_tables),
    check_all: None,
    applies_to: "TTF",
};
