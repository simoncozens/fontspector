use crate::{
    prelude::*,
    Check,
    CheckResult,
    Context,
    FileTypeConvert,
    StatusCode,
};
use font_types::NameId;
use read_fonts::TableProvider;
use serde_json::Map;
use write_fonts::{
    FontBuilder,
    tables::name::{Name, NameRecord}
};

#[macro_export] macro_rules! TEST_FILE {
    ($fname:expr) => {
        Testable::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/test/",
            $fname
        ))
        .unwrap()
    };
}

pub fn run_check(
    check: Check<'_>,
    font: Testable,
) -> std::option::Option<CheckResult> {
    let ctx: Context = Context {
        skip_network: false,
        network_timeout: Some(10),
        configuration: Map::new(),
        check_metadata: check.metadata(),
    };
    check.run(&TestableType::Single(&font), &ctx, None)
}

pub fn assert_pass(check_result: std::option::Option<CheckResult>) {
    let status = check_result.unwrap().worst_status();
    assert_eq!(status, StatusCode::Pass);
}

pub fn assert_results_contain(
    check_result: std::option::Option<CheckResult>,
    severity: StatusCode,
    code: Option<String>,
) {
    let subresults = check_result.unwrap().subresults;
    assert!(subresults
        .iter()
        .any(|subresult| subresult.severity == severity && subresult.code == code));
}

pub fn set_name_entry(font: &mut Testable, platform: u16, encoding: u16, language: u16, nameid: NameId, new_string: String){
    use std::collections::BTreeSet;

    let f = TTF.from_testable(&font).unwrap();
    let name = f.font().name().unwrap();

    let new_record = NameRecord::new(
        platform,
        encoding,
        language,
        nameid,
        new_string.to_string().into(),
    );
    let mut new_records: BTreeSet<NameRecord> = name
        .name_record()
        .iter()
        .filter(|record| record.name_id() != nameid)
        .map(|r| {
            #[allow(clippy::unwrap_used)]
            NameRecord::new(
                r.platform_id(),
                r.encoding_id(),
                r.language_id(),
                r.name_id(),
                r.string(name.string_data())
                    .unwrap()
                    .chars()
                    .collect::<String>()
                    .to_string()
                    .into(),
            )
        })
        .collect();
    new_records.insert(new_record);
    let new_nametable = Name::new(new_records);
    let new_bytes = FontBuilder::new()
        .add_table(&new_nametable)
        .unwrap()
        .copy_missing_tables(f.font())
        .build();

    font.contents = new_bytes;
}
