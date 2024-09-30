use crate::{
    context::Context,
    prelude::FixFnResult,
    status::{CheckError, CheckFnResult},
    testable::{TestableCollection, TestableType},
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
    pub fn applies(&self, f: &'a TestableType, registry: &Registry) -> bool {
        match (&self.implementation, f) {
            (CheckImplementation::CheckAll(_), TestableType::Collection(_)) => true,
            (CheckImplementation::CheckOne(_), TestableType::Single(f)) => registry
                .filetypes
                .get(self.applies_to)
                .map_or(false, |ft| ft.applies(f)),
            _ => false,
        }
    }

    fn clarify_result(
        &'a self,
        fn_result: CheckFnResult,
        file: Option<&'a Testable>,
        section: Option<&str>,
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

    /// Run the check, either on a collection or a single file.
    ///
    /// Returns `None` if the check is not applicable to the given testable.
    pub fn run(
        &'a self,
        testable: &'a TestableType,
        context: &Context,
        section: Option<&str>,
    ) -> Option<CheckResult> {
        match (&self.implementation, testable) {
            (CheckImplementation::CheckAll(_), TestableType::Single(_)) => None,
            (CheckImplementation::CheckOne(_), TestableType::Collection(_)) => None,
            (CheckImplementation::CheckOne(check_one), TestableType::Single(f)) => {
                Some(self.clarify_result(check_one(f, context), Some(f), section))
            }
            (CheckImplementation::CheckAll(check_all), TestableType::Collection(f)) => {
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
