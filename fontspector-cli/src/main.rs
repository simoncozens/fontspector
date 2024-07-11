//! Quality control for OpenType fonts
use clap::Parser;
use fontspector_checkapi::{Check, CheckResult, Plugin, Registry, StatusCode, Testable};
use itertools::iproduct;
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
    Universal
        .register(&mut registry)
        .expect("Couldn't register universal profile, fontspector bug");
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

    for (sectionname, checknames) in profile.sections.iter() {
        println!("Checking section {:}", sectionname);
        let checks: Vec<&Check> = checknames
            .iter()
            .map(|name| {
                let n: &str = name;
                registry.checks.get(n).unwrap()
            })
            .collect();

        let results_all = [];
        // let results_all: Vec<CheckResult> = checks
        //     .iter()
        //     .flat_map(|check| check.run_all(&collection))
        //     .collect();
        let all_checks: Vec<_> = iproduct!(checks.iter(), testables.iter())
            .filter(|(check, file)| check.applies(file, &registry))
            .collect();

        let results_one: Vec<CheckResult> = all_checks
            .par_iter()
            .map(|(check, file)| check.run_one(file))
            .flatten()
            .collect();

        for result in results_all
            .iter()
            .chain(results_one.iter())
            .filter(|c| c.status.code >= args.loglevel)
        {
            println!(">> {:}", result.check_id);
            if args.verbose > 1 {
                println!("   {:}", result.check_name);
            }
            if let Some(filename) = &result.filename {
                println!("   with {:}\n", filename);
            }
            if let Some(rationale) = &result.check_rationale {
                if args.verbose > 1 {
                    termimad::print_inline(&format!("Rationale:\n\n```\n{}\n```\n", rationale));
                }
            }
            termimad::print_inline(&format!("Result: **{:}**\n\n", result.status));
            if result.status.code != StatusCode::Fail {
                continue;
            }
            let check = registry.checks.get(&result.check_id).unwrap();
            if let Some(fix) = check.hotfix {
                if args.hotfix {
                    if fix(&Testable::new(result.filename.as_ref().unwrap())) {
                        // XXX
                        println!("   Hotfix applied");
                    } else {
                        println!("   Hotfix failed");
                    }
                } else {
                    termimad::print_inline("  This issue can be fixed automatically. Run with `--hotfix` to apply the fix.\n")
                }
            }
            println!("\n");
        }
    }
}
