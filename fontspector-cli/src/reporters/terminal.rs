use super::RunResults;
use crate::{reporters::Reporter, Args};
use colored::{ColoredString, Colorize};
use fontspector_checkapi::{FixResult, Registry, StatusCode};
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
    fn report(&self, results: &RunResults, args: &Args, _registry: &Registry) {
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
                            Path::new(filename)
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy(),
                            result.check_id.bright_cyan(),
                            colored_status(result.worst_status(), None),
                            subresults
                                .iter()
                                .map(|r| colored_status(r.severity, r.code.as_deref()))
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
                    println!(">> {:}", result.check_id);
                    if args.verbose > 1 {
                        println!("   {:}", result.check_name);
                        termimad::print_inline(&format!(
                            "Rationale:\n\n```\n{}\n```\n",
                            result.check_rationale
                        ));
                    }
                    for subresult in subresults {
                        termimad::print_inline(&format!("{:}\n", subresult));
                    }
                    match &result.hotfix_result {
                        Some(FixResult::Available) => {
                            termimad::print_inline("  This issue can be fixed automatically. Run with `--hotfix` to apply the fix.\n")
                        }
                        Some(FixResult::Fixed) => {
                            termimad::print_inline("  Hotfix applied.\n")
                        }
                        Some(FixResult::FixError(e)) => {
                            termimad::print_inline(&format!("  Hotfix failed: {:}\n", e))
                        }
                        _ => {}
                    }
                    match &result.sourcefix_result {
                        Some(FixResult::Available) => {
                            termimad::print_inline("  This issue can be fixed by modifying the source. Run with `--fix-sources` to apply the fix.\n")
                        }
                        Some(FixResult::Fixed) => {
                            termimad::print_inline("  Source fix applied.\n")
                        }
                        Some(FixResult::FixError(e)) => {
                            termimad::print_inline(&format!("  Source fix failed: {:}\n", e))
                        }
                        _ => {}
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
