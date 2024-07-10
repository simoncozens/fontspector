use crate::{font::FontCollection, Registry, Status, StatusList, Testable};

pub type CheckId = String;

#[derive(Clone)]
pub struct Check<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub rationale: Option<&'a str>,
    pub proposal: Option<&'a str>,
    pub check_one: Option<&'a dyn Fn(&Testable) -> StatusList>,
    pub check_all: Option<&'a dyn Fn(&FontCollection) -> StatusList>,
    pub hotfix: Option<&'a dyn Fn(&Testable) -> bool>,
    pub fix_source: Option<&'a dyn Fn(&Testable) -> bool>,
    pub applies_to: &'a str,
}

// Are we? Really? I don't know. Let's find out...
unsafe impl Sync for Check<'_> {}

pub struct CheckResult {
    pub status: Status,
    pub check_id: CheckId,
    pub check_name: String,
    pub check_rationale: Option<String>,
    pub filename: Option<String>,
}

impl<'a> Check<'a> {
    pub fn applies(&self, f: &'a Testable, registry: &Registry) -> bool {
        registry
            .filetypes
            .get(self.applies_to)
            .map_or(false, |ft| ft.applies(f))
    }

    pub fn run_one(&'a self, f: &'a Testable) -> Vec<CheckResult> {
        if let Some(check_one) = self.check_one {
            return check_one(f)
                .map(|r| CheckResult {
                    status: r,
                    check_id: self.id.to_string(),
                    check_name: self.title.to_string(),
                    check_rationale: self.rationale.map(|x| x.to_string()),
                    filename: Some(f.filename.clone()),
                })
                .collect();
        }
        vec![]
    }

    pub fn run_all(&'a self, f: &'a FontCollection) -> Vec<CheckResult> {
        if let Some(check_all) = self.check_all {
            check_all(f)
                .map(|r| CheckResult {
                    status: r,
                    check_id: self.id.to_string(),
                    check_name: self.title.to_string(),
                    check_rationale: self.rationale.map(|x| x.to_string()),
                    filename: None,
                })
                .collect()
        } else {
            vec![]
        }
    }
}

pub fn return_result(problems: Vec<Status>) -> Box<dyn Iterator<Item = Status>> {
    if problems.is_empty() {
        Status::just_one_pass()
    } else {
        Box::new(problems.into_iter())
    }
}
