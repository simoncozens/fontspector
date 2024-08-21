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
pub enum CheckImplementation<'a> {
    CheckOne(&'a CheckOneSignature),
    CheckAll(&'a CheckAllSignature),
}

#[derive(Clone)]
pub struct Check<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub rationale: &'a str,
    pub proposal: &'a str,
    pub implementation: CheckImplementation<'a>,
    pub hotfix: Option<&'a dyn Fn(&Testable) -> FixFnResult>,
    pub fix_source: Option<&'a dyn Fn(&Testable) -> FixFnResult>,
    pub applies_to: &'a str,
    pub flags: CheckFlags,
}

// Are we? Really? I don't know. Let's find out...
unsafe impl Sync for Check<'_> {}

impl<'a> Check<'a> {
    pub fn runs_on_collection(&self) -> bool {
        matches!(self.implementation, CheckImplementation::CheckAll(_))
    }
    pub fn applies(&self, f: &'a Testable, registry: &Registry) -> bool {
        registry
            .filetypes
            .get(self.applies_to)
            .map_or(false, |ft| ft.applies(f))
    }

    fn clarify_result(
        &'a self,
        fn_result: CheckFnResult,
        file: Option<&'a Testable>,
        section: &str,
    ) -> CheckResult {
        let subresults = match fn_result {
            Ok(results) => results.collect::<Vec<_>>(),
            Err(CheckError::Error(e)) => vec![Status::error(&format!("Error: {}", e))],
            Err(CheckError::Skip { code, message }) => vec![Status::skip(&code, &message)],
        };
        let res = if subresults.is_empty() {
            vec![Status::pass()]
        } else {
            subresults
        };
        CheckResult::new(self, file.and_then(|f| f.filename.to_str()), section, res)
    }

    pub fn run_one(
        &'a self,
        f: &'a Testable,
        context: &Context,
        section: &str,
    ) -> Option<CheckResult> {
        match self.implementation {
            CheckImplementation::CheckAll(_) => None,
            CheckImplementation::CheckOne(check_one) => {
                Some(self.clarify_result(check_one(f, context), Some(f), section))
            }
        }
    }

    pub fn run_all(
        &'a self,
        f: &'a TestableCollection,
        context: &Context,
        section: &str,
    ) -> Option<CheckResult> {
        match self.implementation {
            CheckImplementation::CheckOne(_) => None,
            CheckImplementation::CheckAll(check_all) => {
                Some(self.clarify_result(check_all(f, context), None, section))
            }
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
