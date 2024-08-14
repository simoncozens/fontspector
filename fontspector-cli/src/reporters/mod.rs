use crate::Args;
use fontspector_checkapi::{CheckResult, Registry, StatusCode, Testable};
use std::collections::HashMap;

pub(crate) mod terminal;
pub(crate) mod json;

pub type RunResults<'a> = Vec<(&'a String, &'a &'a Testable, Vec<CheckResult>)>;

pub fn worst_status(results: &RunResults) -> StatusCode {
    results
        .iter()
        .map(|(_sectionname, _testable, checkresults)| {
            checkresults
                .iter()
                .map(|r| r.status.code)
                .max()
                .unwrap_or(StatusCode::Pass)
        })
        .max()
        .unwrap_or(StatusCode::Pass)
}

pub type OrganisedResults<'a> = HashMap<&'a Testable, HashMap<String, Vec<CheckResult>>>;

/// Organize the results by testable and section
pub fn organize(results: RunResults) -> OrganisedResults {
    let mut organised_results: OrganisedResults = HashMap::new();
    for (sectionname, testable, checkresults) in results {
        // let filename = testable.filename.clone();
        let section = organised_results.entry(testable).or_default();
        let results = section.entry(sectionname.clone()).or_default();
        results.extend(checkresults);
    }
    organised_results
}

pub fn summary_results(organised_results: &OrganisedResults) -> HashMap<StatusCode, i32> {
    let mut summary = HashMap::new();
    for (_testable, sectionresults) in organised_results.iter() {
        for (_sectionname, results) in sectionresults.iter() {
            for result in results.iter() {
                let entry = summary.entry(result.status.code).or_insert(0);
                *entry += 1;
            }
        }
    }
    summary
}

pub trait Reporter {
    fn report(&self, organised_results: &OrganisedResults, args: &Args, registry: &Registry);
}
