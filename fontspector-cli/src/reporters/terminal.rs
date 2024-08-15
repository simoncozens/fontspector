use super::RunResults;
use crate::{reporters::Reporter, Args};
use colored::{ColoredString, Colorize};
use fontspector_checkapi::{Registry, StatusCode, Testable};
use itertools::Itertools;
use std::{collections::HashMap, path::Path};

pub(crate) struct TerminalReporter {
    succinct: bool,
}

impl TerminalReporter {
    pub fn new(succinct: bool) -> Self {
        Self { succinct }
    }
}

fn colored_status(c: StatusCode, s: Option<&str>) -> ColoredString {
    let string = match s {
        Some(s) => s.to_string(),
        None => c.to_string(),
    };
    match c {
        StatusCode::Error => string.on_red(),
        StatusCode::Fail => string.red(),
        StatusCode::Warn => string.yellow(),
        StatusCode::Info => string.cyan(),
        StatusCode::Skip => string.blue(),
        StatusCode::Pass => string.green(),
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
                    let subresults = result
                        .subresults
                        .iter()
                        .filter(|c| c.severity >= args.loglevel)
                        .collect::<Vec<_>>();
                    if subresults.is_empty() {
                        continue;
                    }

                    if self.succinct {
                        println!(
                            "{:}: {:} {:} [{}]",
                            Path::new(filename).file_name().unwrap().to_string_lossy(),
                            result.check_id.bright_cyan(),
                            colored_status(result.worst_status(), None),
                            subresults
                                .iter()
                                .map(|r| colored_status(
                                    r.severity,
                                    r.code.as_ref().map(|x| x.as_str())
                                ))
                                .join(" ")
                        );
                        continue;
                    }

                    if !fileheading_done {
                        println!("Testing: {:}", filename);
                        fileheading_done = true;
                    }
                    if !sectionheading_done {
                        println!("  Section: {:}\n", sectionname);
                        sectionheading_done = true;
                    }
                    for subresult in subresults {
                        println!(">> {:}", result.check_id);
                        if args.verbose > 1 {
                            println!("   {:}", result.check_name);
                            termimad::print_inline(&format!(
                                "Rationale:\n\n```\n{}\n```\n",
                                result.check_rationale
                            ));
                        }
                        termimad::print_inline(&format!("{:}\n", subresult));
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

impl TerminalReporter {
    pub fn summary_report(summary: HashMap<StatusCode, i32>) {
        print!("\nSummary:\n  ");
        for code in StatusCode::all() {
            print!(
                "{:}: {:} ",
                colored_status(code, None),
                summary.get(&code).unwrap_or(&0)
            );
        }
        println!();
    }
}
