use std::collections::HashSet;

// Note that most of this fontbakery check is folded into unwanted_tables.
// All that's left is the check for zz* feature tags and language systems.
use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;

#[check(
    id = "vtt_volt_data",
    title = "VTT or Volt Source Data must not be present.",
    rationale = "
        Check to make sure all the VTT source (TSI* tables) and
        VOLT stuff (TSIV and zz features & langsys records) are gone.",
    proposal = "https://github.com/fonttools/fontbakery/pull/4657"
)]
fn vtt_volt_data(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    let bad_tags = f
        .feature_records(false)
        .map(|(f, _)| f.feature_tag().to_string())
        .filter(|tag| tag.starts_with("zz"))
        .collect::<Vec<_>>();
    if !bad_tags.is_empty() {
        problems.push(Status::fail(
            "volt-feature",
            &format!(
                "Found unwanted VOLT feature tags:\n{}",
                bullet_list(context, bad_tags)
            ),
        ));
    }
    // Language systems, urgh
    let mut bad_langsys = HashSet::new();
    if let Ok(gsub) = f.font().gsub() {
        if let Ok(script_list) = gsub.script_list() {
            for script in script_list.script_records() {
                let s = script.script(script_list.offset_data())?;
                bad_langsys.extend(
                    s.lang_sys_records()
                        .iter()
                        .map(|ls| ls.lang_sys_tag().to_string())
                        .filter(|tag| tag.starts_with("zz")),
                );
            }
        }
    }
    if let Ok(gpos) = f.font().gpos() {
        if let Ok(script_list) = gpos.script_list() {
            for script in script_list.script_records() {
                let s = script.script(script_list.offset_data())?;
                bad_langsys.extend(
                    s.lang_sys_records()
                        .iter()
                        .map(|ls| ls.lang_sys_tag().to_string())
                        .filter(|tag| tag.starts_with("zz")),
                );
            }
        }
    }
    if !bad_langsys.is_empty() {
        problems.push(Status::fail(
            "volt-langsys",
            &format!(
                "Found unwanted VOLT language system tags:\n{}",
                bullet_list(context, bad_langsys.iter())
            ),
        ));
    }
    return_result(problems)
}
