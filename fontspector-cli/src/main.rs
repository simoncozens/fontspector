#![deny(clippy::unwrap_used, clippy::expect_used)]
//! Quality control for OpenType fonts

mod args;
mod reporters;

use args::Args;
use clap::Parser;
use fontspector_checkapi::{Check, Context, Plugin, Registry, Testable};
use indicatif::ParallelProgressIterator;
use profile_googlefonts::GoogleFonts;
use profile_universal::Universal;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use reporters::{
    organize, summary_results, terminal::TerminalReporter, worst_status, Reporter, RunResults,
};
use serde_json::Map;

/// Filter out checks that don't apply
fn included_excluded(checkname: &str, args: &Args) -> bool {
    if let Some(checkids) = &args.checkid {
        if !checkids.iter().any(|id| checkname.contains(id)) {
            return false;
        }
    }
    if let Some(exclude_checkids) = &args.exclude_checkid {
        if exclude_checkids.iter().any(|id| checkname.contains(id)) {
            return false;
        }
    }
    true
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
    for plugin_path in args.plugins.iter() {
        if let Err(err) = registry.load_plugin(plugin_path) {
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

    if testables.is_empty() {
        log::error!("No input files");
        std::process::exit(1);
    }

    // Load configuration
    let configuration: Map<String, serde_json::Value> = args
        .configuration
        .as_ref()
        .map(|filename| {
            std::fs::File::open(filename).unwrap_or_else(|e| {
                println!("Could not open configuration file {}: {:}", filename, e);
                std::process::exit(1)
            })
        })
        .and_then(|file| {
            serde_json::from_reader(std::io::BufReader::new(file)).unwrap_or_else(|e| {
                println!("Could not parse configuration file: {:}", e);
                std::process::exit(1)
            })
        })
        .map(|file: serde_json::Value| {
            file.as_object()
                .unwrap_or_else(|| {
                    println!("Configuration file must be a JSON object");
                    std::process::exit(1)
                })
                .clone()
        })
        .unwrap_or_default();

    // Establish a check order
    let checkorder: Vec<(String, &Testable, &Check, Context)> = profile
        .sections
        .iter()
        .flat_map(|(sectionname, checknames)| {
            #[allow(clippy::unwrap_used)] // We previously ensured the check exists in the registry
            checknames
                .iter()
                .filter(|checkname| included_excluded(checkname, &args))
                .map(|checkname| {
                    (
                        sectionname.clone(),
                        registry.checks.get(checkname).unwrap(),
                        context_for(checkname, &args, &configuration),
                    )
                })
        })
        .flat_map(|(sectionname, check, context): (String, &Check, Context)| {
            testables
                .iter()
                .filter(|testable| check.applies(testable, &registry))
                .map(move |testable| (sectionname.clone(), testable, check, context.clone()))
        })
        .collect();

    if !args.quiet {
        println!(
            "Running {:} check{} on {:} file{}",
            checkorder.len(),
            if checkorder.len() == 1 { "" } else { "s" },
            testables.len(),
            if testables.len() == 1 { "" } else { "s" }
        );
    }

    let results: RunResults = checkorder
        .par_iter()
        .progress()
        .map(|(sectionname, testable, check, context)| {
            (sectionname, testable, check.run_one(testable, context))
        })
        .collect();

    let worst_status = worst_status(&results);
    let organised_results = organize(results);

    let mut reporters: Vec<Box<dyn Reporter>> = vec![];
    if !args.quiet {
        reporters.push(Box::new(TerminalReporter {}));
    }

    for reporter in reporters {
        reporter.report(&organised_results, &args, &registry);
    }

    if !args.quiet {
        // Summary report
        let summary = summary_results(organised_results);
        print!("\nSummary:\n  ");
        for (status, count) in summary.iter() {
            print!("{:}: {:}  ", status, count);
        }
        println!();
    }
    if worst_status >= args.error_code_on {
        std::process::exit(1);
    }
}

fn context_for(
    checkname: &str,
    args: &Args,
    configuration: &Map<String, serde_json::Value>,
) -> Context {
    Context {
        skip_network: args.skip_network,
        network_timeout: args.timeout,
        configuration: configuration
            .get(checkname)
            .and_then(|x| x.as_object())
            .cloned()
            .unwrap_or_default(),
    }
}
