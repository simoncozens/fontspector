#![deny(clippy::unwrap_used, clippy::expect_used)]
//! Quality control for OpenType fonts
use std::collections::HashMap;

use clap::Parser;
use fontspector_checkapi::{Check, CheckResult, Plugin, Registry, StatusCode, Testable};
use indicatif::ParallelProgressIterator;
use itertools::Itertools;
// use rayon::prelude::*;

use profile_googlefonts::GoogleFonts;
use profile_universal::Universal;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
/// Quality control for OpenType fonts
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Increase logging
    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,

    /// Hotfix
    #[clap(short, long)]
    hotfix: bool,

    /// Log level
    #[clap(short, long, arg_enum, value_parser, default_value_t=StatusCode::Warn)]
    loglevel: StatusCode,

    /// Plugins to load
    #[clap(long, value_delimiter = ',')]
    plugins: Vec<String>,

    /// Profile to check
    #[clap(short, long, default_value = "universal")]
    profile: String,

    /// Input files
    inputs: Vec<String>,
}

fn main() {
    // Command line handling
    let args = Args::parse();

    env_logger::init_from_env(env_logger::Env::default().filter_or(
        env_logger::DEFAULT_FILTER_ENV,
        match args.verbose {
            0 => "warn",
            1 => "info",
            _ => "debug",
        },
    ));

    // Set up the check registry
    let mut registry = Registry::new();
    #[allow(clippy::expect_used)] // If this fails, I *want* to panic
    Universal
        .register(&mut registry)
        .expect("Couldn't register universal profile, fontspector bug");
    #[allow(clippy::expect_used)] // If this fails, I *want* to panic
    GoogleFonts
        .register(&mut registry)
        .expect("Couldn't register googlefonts profile, fontspector bug");
    for plugin_path in args.plugins {
        if let Err(err) = registry.load_plugin(&plugin_path) {
            log::error!("Could not load plugin {:}: {:}", plugin_path, err);
        }
    }

    // Load the relevant profile
    let profile = registry.get_profile(&args.profile).unwrap_or_else(|| {
        log::error!("Could not find profile {:}", args.profile);
        std::process::exit(1);
    });
    let testables: Vec<Testable> = args.inputs.iter().map(|x| Testable::new(x)).collect();
    // let collection = FontCollection(thing);

    // Establish a check order
    let checkorder: Vec<(String, &Testable, &Check)> = profile
        .sections
        .iter()
        .flat_map(|(sectionname, checknames)| {
            #[allow(clippy::unwrap_used)] // We previously ensured the check exists in the registry
            checknames
                .iter()
                .map(|checkname| (sectionname.clone(), registry.checks.get(checkname).unwrap()))
        })
        .flat_map(|(sectionname, check)| {
            testables
                .iter()
                .filter(|testable| check.applies(testable, &registry))
                .map(move |testable| (sectionname.clone(), testable, check))
        })
        .collect();

    println!("Testing...");
    let results: Vec<_> = checkorder
        .par_iter()
        .progress()
        .map(|(sectionname, testable, check)| (sectionname, testable, check.run_one(testable)))
        .collect();

    // Organise results by testable and sectionname
    let mut organised_results: HashMap<&Testable, HashMap<String, Vec<CheckResult>>> =
        HashMap::new();
    for (sectionname, testable, checkresults) in results {
        // let filename = testable.filename.clone();
        let section = organised_results.entry(testable).or_default();
        let results = section.entry(sectionname.clone()).or_default();
        results.extend(checkresults);
    }

    for (testable, sectionresults) in organised_results
        .iter()
        .sorted_by_key(|(t, _s)| &t.filename)
    {
        let mut fileheading_done = false;
        for (sectionname, results) in sectionresults.iter() {
            let mut sectionheading_done = false;
            for result in results.iter().filter(|c| c.status.code >= args.loglevel) {
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
                }
                if args.verbose > 1 {
                    termimad::print_inline(&format!("Rationale:\n\n```\n{}\n```\n", result.check_rationale));
                }
                termimad::print_inline(&format!("**{:}**", result.status));
                if result.status.code != StatusCode::Fail {
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
