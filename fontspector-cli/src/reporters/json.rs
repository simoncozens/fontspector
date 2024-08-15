use crate::{
    reporters::{Reporter, RunResults},
    Args,
};
use fontspector_checkapi::Registry;
use serde::Serialize;
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
                        .serialize(serde_json::value::Serializer)
                        .unwrap_or_else(|e| {
                            log::error!("Error serializing JSON report: {:}", e);
                            std::process::exit(1);
                        }),
                );
            }
            results.insert(testable.clone(), testable_result.into());
        }
        let output = json!({
            "summary": summary,
            "results": results,
        });

        let report = serde_json::to_string_pretty(&output).unwrap_or_else(|e| {
            log::error!("Error serializing JSON report: {:}", e);
            std::process::exit(1);
        });

        std::fs::write(&self.filename, report).unwrap_or_else(|e| {
            log::error!("Error writing JSON report to {:}: {:}", self.filename, e);
            std::process::exit(1);
        });
    }
}
