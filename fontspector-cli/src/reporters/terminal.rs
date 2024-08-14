use fontspector_checkapi::{CheckResult, Registry, StatusCode, Testable};
use itertools::Itertools;
use std::collections::HashMap;

use crate::{reporters::Reporter, Args};
pub(crate) struct TerminalReporter {
    succinct: bool
}

impl TerminalReporter {
    pub fn new(succinct: bool) -> Self {
        Self { succinct }
    }
}

impl Reporter for TerminalReporter {
    fn report(
        &self,
        organised_results: &HashMap<&Testable, HashMap<String, Vec<CheckResult>>>,
        args: &Args,
        registry: &Registry,
    ) {
        for (testable, sectionresults) in organised_results
            .iter()
            .sorted_by_key(|(t, _s)| &t.filename)
        {
            let mut fileheading_done = false;
            for (sectionname, results) in sectionresults.iter() {
                let mut sectionheading_done = false;
                for result in results.iter().filter(|c| c.status.code >= args.loglevel) {
                    if self.succinct {
                        println!("{:}: {:} {:}",
                            testable.filename,
                            result.check_id,
                            result.status.code,
                        );
                        continue
                    }
                    if !fileheading_done {
                        println!("Testing: {:}", testable.filename);
                        fileheading_done = true;
                    }
                    if !sectionheading_done {
                        println!("  Section: {:}\n", sectionname);
                        sectionheading_done = true;
                    }
                    println!(">> {:}", result.check_id);
                    if args.verbose > 1 {
                        println!("   {:}", result.check_name);
                        termimad::print_inline(&format!(
                            "Rationale:\n\n```\n{}\n```\n",
                            result.check_rationale
                        ));
                    }
                    termimad::print_inline(&format!("{:}\n", result.status));
                    if result.status.code != StatusCode::Fail {
                        println!();
                        continue;
                    }
                    #[allow(clippy::unwrap_used)]
                    // This is a genuine can't-happen. We put it in the hashmap earlier!
                    let check = registry.checks.get(&result.check_id).unwrap();
                    if let Some(fix) = check.hotfix {
                        if args.hotfix {
                            match fix(testable) {
                                Ok(true) => println!("   Hotfix applied"),
                                Ok(false) => println!("   Hotfix not applied"),
                                Err(e) => println!("   Hotfix failed: {:}", e),
                            }
                        } else {
                            termimad::print_inline("  This issue can be fixed automatically. Run with `--hotfix` to apply the fix.\n")
                        }
                    }
                    println!("\n");
                }
            }
        }
    }
}
