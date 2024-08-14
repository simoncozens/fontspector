use fontspector_checkapi::{Registry, StatusCode, Testable};
use itertools::Itertools;

use crate::{reporters::Reporter, Args};

use super::RunResults;
pub(crate) struct TerminalReporter {
    succinct: bool,
}

impl TerminalReporter {
    pub fn new(succinct: bool) -> Self {
        Self { succinct }
    }
}

impl Reporter for TerminalReporter {
    fn report(&self, results: &RunResults, args: &Args, registry: &Registry) {
        let organised_results = results.organize();
        for (filename, sectionresults) in organised_results
            .iter()
            .sorted_by_key(|(t, _s)| t.to_string())
        {
            let mut fileheading_done = false;
            for (sectionname, results) in sectionresults.iter() {
                let mut sectionheading_done = false;
                for result in results.iter() {
                    if self.succinct {
                        println!(
                            "{:}: {:} {:}",
                            filename,
                            result.check_id,
                            result
                                .subresults
                                .iter()
                                .flat_map(|r| r.code.as_ref())
                                .join(", ")
                        );
                        continue;
                    }

                    for subresult in result
                        .subresults
                        .iter()
                        .filter(|c| c.severity >= args.loglevel)
                    {
                        if !fileheading_done {
                            println!("Testing: {:}", filename);
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
                        termimad::print_inline(&format!("{:}\n", subresult));
                        if subresult.severity != StatusCode::Fail {
                            println!();
                            continue;
                        }
                        #[allow(clippy::unwrap_used)]
                        // This is a genuine can't-happen. We put it in the hashmap earlier!
                        let check = registry.checks.get(&result.check_id).unwrap();
                        if let Some(fix) = check.hotfix {
                            if args.hotfix {
                                match fix(&Testable::new(filename)) {
                                    Ok(true) => println!("   Hotfix applied"),
                                    Ok(false) => println!("   Hotfix not applied"),
                                    Err(e) => println!("   Hotfix failed: {:}", e),
                                }
                            } else {
                                termimad::print_inline("  This issue can be fixed automatically. Run with `--hotfix` to apply the fix.\n")
                            }
                        }
                    }
                    println!("\n");
                }
            }
        }
    }
}
