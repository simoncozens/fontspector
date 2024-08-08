use crate::{
    font::FontCollection, prelude::FixFnResult, status::CheckFnResult, Registry, Status, Testable,
};

pub type CheckId = String;

#[derive(Clone)]
pub struct Check<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub rationale: Option<&'a str>,
    pub proposal: Option<&'a str>,
    pub check_one: Option<&'a dyn Fn(&Testable) -> CheckFnResult>,
    pub check_all: Option<&'a dyn Fn(&FontCollection) -> CheckFnResult>,
    pub hotfix: Option<&'a dyn Fn(&Testable) -> FixFnResult>,
    pub fix_source: Option<&'a dyn Fn(&Testable) -> FixFnResult>,
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

    fn status_to_result(&'a self, status: Status, file: Option<&'a Testable>) -> CheckResult {
        CheckResult {
            status,
            check_id: self.id.to_string(),
            check_name: self.title.to_string(),
            check_rationale: self.rationale.map(|x| x.to_string()),
            filename: file.map(|x| x.filename.clone()),
        }
    }

    pub fn run_one(&'a self, f: &'a Testable) -> Vec<CheckResult> {
        if let Some(check_one) = self.check_one {
            match check_one(f) {
                Ok(results) => results.map(|r| self.status_to_result(r, Some(f))).collect(),
                Err(e) => {
                    vec![self.status_to_result(Status::error(&format!("Error: {}", e)), Some(f))]
                }
            }
        } else {
            vec![]
        }
    }

    pub fn run_all(&'a self, f: &'a FontCollection) -> Vec<CheckResult> {
        if let Some(check_all) = self.check_all {
            match check_all(f) {
                Ok(results) => results.map(|r| self.status_to_result(r, None)).collect(),
                Err(e) => {
                    vec![self.status_to_result(Status::error(&format!("Error: {}", e)), None)]
                }
            }
        } else {
            vec![]
        }
    }
}

pub fn return_result(problems: Vec<Status>) -> CheckFnResult {
    if problems.is_empty() {
        Ok(Status::just_one_pass())
    } else {
        Ok(Box::new(problems.into_iter()))
    }
}
