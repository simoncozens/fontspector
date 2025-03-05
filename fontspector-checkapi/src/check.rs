use std::time::Duration;

use crate::{
    context::Context,
    prelude::FixFnResult,
    status::{CheckError, CheckFnResult},
    testable::{TestableCollection, TestableType},
    CheckResult, Registry, Status, Testable,
};

/// A check ID is a unique identifier for a check
pub type CheckId = String;
/// The function signature for a check taking a single testable
type CheckOneSignature = dyn Fn(&Testable, &Context) -> CheckFnResult;
/// The function signature for a check taking a collection of testables
type CheckAllSignature = dyn Fn(&TestableCollection, &Context) -> CheckFnResult;

#[derive(Clone)]
/// Additional flags added to a check
pub struct CheckFlags {
    /// Whether the check is experimental
    pub experimental: bool,
}

impl CheckFlags {
    // We can't use Default trait here because we want to use
    // it in const context.
    /// Create a new CheckFlags with default values
    pub const fn default() -> Self {
        Self {
            experimental: false,
        }
    }
}

#[derive(Clone)]
/// A check definition
///
/// This wraps a check function which may take either a single file, or
/// a collection of files. The check function is wrapped in an enum to
/// unify the different signatures.
pub enum CheckImplementation<'a> {
    /// A check that takes a single file
    CheckOne(&'a CheckOneSignature),
    /// A check that takes a collection of files
    CheckAll(&'a CheckAllSignature),
}

/// The function signature for a hotfix function
pub type HotfixFunction = dyn Fn(&mut Testable) -> FixFnResult;

#[derive(Clone)]
/// A check definition
pub struct Check<'a> {
    /// The check's unique identifier
    pub id: &'a str,
    /// Title to be displayed to the user
    pub title: &'a str,
    /// A short description of the check
    pub rationale: &'a str,
    /// URL where the check was proposed
    pub proposal: &'a str,
    /// Function pointer implementing the actual check
    pub implementation: CheckImplementation<'a>,
    /// Function pointer implementing a hotfix to the binary file
    pub hotfix: Option<&'a HotfixFunction>,
    /// Function pointer implementing a hotfix to the font source file
    pub fix_source: Option<&'a dyn Fn(&Testable) -> FixFnResult>,
    /// A registered file type that this check applies to
    pub applies_to: &'a str,
    /// Additional flags for the check
    pub flags: CheckFlags,
    /// Metadata for the check in JSON format
    pub _metadata: Option<&'static str>,
}

// Are we? Really? I don't know. Let's find out...
unsafe impl Sync for Check<'_> {}

impl<'a> Check<'a> {
    /// Does this check run on a collection of files?
    pub fn runs_on_collection(&self) -> bool {
        matches!(self.implementation, CheckImplementation::CheckAll(_))
    }
    /// Should the check run on the given testable?
    ///
    /// Checks declare themselves to either run on a collection of files, or
    /// on a single file of a given file type; this function checks if the
    /// check is applicable to the given testable.
    pub fn applies(&self, f: &'a TestableType, registry: &Registry) -> bool {
        match (&self.implementation, f) {
            (CheckImplementation::CheckAll(_), TestableType::Collection(_)) => true,
            (CheckImplementation::CheckOne(_), TestableType::Single(f)) => registry
                .filetypes
                .get(self.applies_to)
                .is_some_and(|ft| ft.applies(f)),
            _ => false,
        }
    }

    /// Get the metadata for this check
    ///
    /// Each check definition can declare associated metadata; this is
    /// interpreted as a JSON string and returned as a serde_json::Value.
    pub fn metadata(&self) -> serde_json::Value {
        #[allow(clippy::expect_used)]
        self._metadata
            .map(|s| serde_json::from_str(s).unwrap_or_else(|_| panic!("Bad JSON in {}", self.id)))
            .unwrap_or_default()
    }

    /// Clarify the result of a check function
    ///
    /// Wraps the bare result with additional metadata identifying the check,
    /// the file, etc. so that it can be reported back to the user.
    fn clarify_result(
        &'a self,
        fn_result: CheckFnResult,
        filename: Option<&str>,
        source_filename: Option<&str>,
        section: Option<&str>,
        duration: Duration,
    ) -> CheckResult {
        let subresults = match fn_result {
            Ok(results) => results.collect::<Vec<_>>(),
            Err(CheckError::Error(e)) => vec![Status::error(None, &format!("Error: {}", e))],
            Err(CheckError::Skip { code, message }) => vec![Status::skip(&code, &message)],
        };
        let res = if subresults.is_empty() {
            vec![Status::pass()]
        } else {
            subresults
        };
        CheckResult::new(self, filename, source_filename, section, res, duration)
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
        // log::debug!("Running check {} on {:?}", self.id, testable);
        match (&self.implementation, testable) {
            (CheckImplementation::CheckAll(_), TestableType::Single(_)) => None,
            (CheckImplementation::CheckOne(_), TestableType::Collection(_)) => None,
            (CheckImplementation::CheckOne(check_one), TestableType::Single(f)) => {
                #[cfg(not(target_family = "wasm"))]
                let start = std::time::Instant::now();
                let result = check_one(f, context);

                #[cfg(not(target_family = "wasm"))]
                let duration = start.elapsed();
                #[cfg(target_family = "wasm")]
                let duration = Duration::from_secs(0);

                Some(self.clarify_result(
                    result,
                    f.filename.to_str(),
                    f.source.as_ref().and_then(|x| x.to_str()),
                    section,
                    duration,
                ))
            }
            (CheckImplementation::CheckAll(check_all), TestableType::Collection(f)) => {
                #[cfg(not(target_family = "wasm"))]
                let start = std::time::Instant::now();
                let result = check_all(f, context);
                #[cfg(not(target_family = "wasm"))]
                let duration = start.elapsed();
                #[cfg(target_family = "wasm")]
                let duration = Duration::from_secs(0);

                Some(self.clarify_result(result, Some(&f.directory), None, section, duration))
            }
        }
    }
}

/// Utility function for returning a check result
///
/// Interprets the case of an empty list of problems to mean a PASS status.
pub fn return_result(problems: Vec<Status>) -> CheckFnResult {
    if problems.is_empty() {
        Ok(Status::just_one_pass())
    } else {
        Ok(Box::new(problems.into_iter()))
    }
}
