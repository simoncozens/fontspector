use crate::{
    context::Context,
    prelude::FixFnResult,
    status::{CheckError, CheckFnResult},
    testable::TestableCollection,
    CheckResult, Registry, Status, Testable,
};

pub type CheckId = String;
type CheckOneSignature = dyn Fn(&Testable, &Context) -> CheckFnResult;
type CheckAllSignature = dyn Fn(&TestableCollection, &Context) -> CheckFnResult;

#[derive(Clone)]
pub struct CheckFlags {
    pub experimental: bool,
}

impl CheckFlags {
    // We can't use Default trait here because we want to use
    // it in const context.
    pub const fn default() -> Self {
        Self {
            experimental: false,
        }
    }
}

#[derive(Clone)]
pub struct Check<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub rationale: &'a str,
    pub proposal: &'a str,
    pub check_one: Option<&'a CheckOneSignature>,
    pub check_all: Option<&'a CheckAllSignature>,
    pub hotfix: Option<&'a dyn Fn(&Testable) -> FixFnResult>,
    pub fix_source: Option<&'a dyn Fn(&Testable) -> FixFnResult>,
    pub applies_to: &'a str,
    pub flags: CheckFlags,
}

// Are we? Really? I don't know. Let's find out...
unsafe impl Sync for Check<'_> {}

impl<'a> Check<'a> {
    pub fn applies(&self, f: &'a Testable, registry: &Registry) -> bool {
        registry
            .filetypes
            .get(self.applies_to)
            .map_or(false, |ft| ft.applies(f))
    }

    fn status_to_result(
        &'a self,
        subresults: Vec<Status>,
        file: Option<&'a Testable>,
        section: &str,
    ) -> CheckResult {
        CheckResult::new(
            self,
            file.and_then(|f| f.filename.to_str()),
            section,
            subresults,
        )
    }

    pub fn run_one(
        &'a self,
        f: &'a Testable,
        context: &Context,
        section: &str,
    ) -> Option<CheckResult> {
        self.check_one.map(|check_one| {
            let subresults = match check_one(f, context) {
                Ok(results) => results.collect::<Vec<_>>(),
                Err(CheckError::Error(e)) => vec![Status::error(&format!("Error: {}", e))],
                Err(CheckError::Skip { code, message }) => vec![Status::skip(&code, &message)],
            };
            self.status_to_result(
                if subresults.is_empty() {
                    Status::just_one_pass().collect()
                } else {
                    subresults
                },
                Some(f),
                section,
            )
        })
    }

    /// XXX This repeated code is horrible.
    pub fn run_all(
        &'a self,
        f: &'a TestableCollection,
        context: &Context,
        section: &str,
    ) -> Option<CheckResult> {
        self.check_all.map(|check_all| {
            let subresults = match check_all(f, context) {
                Ok(results) => results.collect::<Vec<_>>(),
                Err(CheckError::Error(e)) => vec![Status::error(&format!("Error: {}", e))],
                Err(CheckError::Skip { code, message }) => vec![Status::skip(&code, &message)],
            };

            self.status_to_result(
                if subresults.is_empty() {
                    Status::just_one_pass().collect()
                } else {
                    subresults
                },
                None,
                section,
            )
        })
    }
}

pub fn return_result(problems: Vec<Status>) -> CheckFnResult {
    if problems.is_empty() {
        Ok(Status::just_one_pass())
    } else {
        Ok(Box::new(problems.into_iter()))
    }
}
