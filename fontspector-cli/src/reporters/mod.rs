use crate::Args;
use fontspector_checkapi::{CheckResult, Registry, StatusCode};
use std::collections::HashMap;

pub(crate) mod json;
pub(crate) mod markdown;
pub(crate) mod terminal;

/// The results of all checks in a check run
pub struct RunResults {
    results: Vec<CheckResult>,
}

impl RunResults {
    /// Iterate over each check
    pub fn iter(&self) -> impl Iterator<Item = &CheckResult> {
        self.results.iter()
    }
    /// Iterate over each check mutably
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut CheckResult> {
        self.results.iter_mut()
    }

    /// Get the worst status of all checks
    pub fn worst_status(&self) -> StatusCode {
        self.results
            .iter()
            .map(|r| r.worst_status())
            .max()
            .unwrap_or(StatusCode::Pass)
    }

    /// Get a summary of the results by status code
    pub fn summary(&self) -> HashMap<StatusCode, i32> {
        let mut summary = HashMap::new();
        for result in self.results.iter() {
            for subresult in result.subresults.iter() {
                let entry = summary.entry(subresult.severity).or_insert(0);
                *entry += 1;
            }
        }
        summary
    }

    /// Organize the results by testable and section
    pub fn organize(&self) -> OrganisedResults {
        let mut organised_results: OrganisedResults = HashMap::new();
        for checkresult in self.iter() {
            let section = organised_results
                .entry(
                    checkresult
                        .filename
                        .clone()
                        .unwrap_or("All fonts".to_string()),
                )
                .or_default();
            let results = section
                .entry(
                    checkresult
                        .section
                        .clone()
                        .unwrap_or("No section".to_string()),
                )
                .or_default();
            results.push(checkresult.clone());
        }
        organised_results
    }

    pub fn len(&self) -> usize {
        self.results.len()
    }
}

impl From<Vec<CheckResult>> for RunResults {
    fn from(val: Vec<CheckResult>) -> Self {
        RunResults { results: val }
    }
}

pub type OrganisedResults<'a> = HashMap<String, HashMap<String, Vec<CheckResult>>>;

pub trait Reporter {
    fn report(&self, organised_results: &RunResults, args: &Args, registry: &Registry);
}
