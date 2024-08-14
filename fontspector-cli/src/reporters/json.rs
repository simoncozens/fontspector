use crate::reporters::{Reporter, RunResults};
use crate::Args;
use fontspector_checkapi::Registry;
use serde_json::{json, Map};
pub(crate) struct JsonReporter {
    filename: String,
}

impl JsonReporter {
    pub fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
        }
    }
}
impl Reporter for JsonReporter {
    fn report(&self, results: &RunResults, _args: &Args, _registry: &Registry) {
        let summary = results.summary();
        let organised_results = results.organize();
        let mut results = Map::new();
        for (testable, sectionresults) in organised_results.iter() {
            let mut testable_result = Map::new();
            for (sectionname, checkresults) in sectionresults.iter() {
                testable_result.insert(
                    sectionname.clone(),
                    checkresults
                        .iter()
                        .map(|r| {
                            json!({
                                "check_id": r.check_id,
                                "check_name": r.check_name,
                                "check_rationale": r.check_rationale,
                                "subresults": r.subresults
                                    .iter()
                                    .map(|r| {
                                        json!({
                                            "status": r.severity.to_string(),
                                            "code": r.code,
                                            "status_message": r.message,
                                        })
                                    })
                                    .collect::<Vec<_>>(),
                            })
                        })
                        .collect(),
                );
            }
            results.insert(testable.clone(), testable_result.into());
        }
        let output = json!({
            "summary": summary,
            "results": results,
        });

        let report = serde_json::to_string_pretty(&output).unwrap();
        // Write to self.filename
        std::fs::write(&self.filename, report).unwrap_or_else(|e| {
            eprintln!("Error writing JSON report to {:}: {:}", self.filename, e);
            std::process::exit(1);
        });
    }
}
