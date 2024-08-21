#![deny(clippy::unwrap_used, clippy::expect_used)]
//! Quality control for OpenType fonts

mod args;
mod reporters;

use std::path::PathBuf;

use args::Args;
use clap::Parser;
use fontspector_checkapi::{
    Check, CheckResult, Context, FixResult, Plugin, Registry, Testable, TestableCollection,
};
use indicatif::ParallelProgressIterator;
use profile_googlefonts::GoogleFonts;
use profile_universal::Universal;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use reporters::{
    json::JsonReporter, markdown::MarkdownReporter, terminal::TerminalReporter, Reporter,
    RunResults,
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

    // We create one collection for each set of testable files in a directory.
    // So let's group the inputs per directory, and then map them into a FontCollection
    let grouped_inputs: Vec<TestableCollection> = args
        .inputs
        .iter()
        .map(|x| PathBuf::from(x))
        .fold(Vec::new(), |mut acc: Vec<Vec<PathBuf>>, path| {
            let directory = path.parent().unwrap().to_path_buf();
            if let Some(group) = acc
                .iter_mut()
                .find(|group| group[0].parent() == Some(&directory))
            {
                group.push(path);
            } else {
                acc.push(vec![path]);
            }
            acc
        })
        .into_iter()
        .map(|group| {
            TestableCollection::from_filenames(&group).unwrap_or_else(|e| {
                log::error!("Could not load files from {:?}: {:}", group[0].parent(), e);
                std::process::exit(1)
            })
        })
        .collect();

    // This is wrong wrong wrong, but let it be for now.
    let testables: Vec<Testable> = grouped_inputs
        .into_iter()
        .flat_map(|collection| collection.testables)
        .collect();

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
                log::error!("Could not open configuration file {}: {:}", filename, e);
                std::process::exit(1)
            })
        })
        .and_then(|file| {
            serde_json::from_reader(std::io::BufReader::new(file)).unwrap_or_else(|e| {
                log::error!("Could not parse configuration file: {:}", e);
                std::process::exit(1)
            })
        })
        .map(|file: serde_json::Value| {
            file.as_object()
                .unwrap_or_else(|| {
                    log::error!("Configuration file must be a JSON object");
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

    let apply_fixes = |(testable, check, mut result): (&&Testable, &&Check, CheckResult)| {
        if args.hotfix {
            if let Some(fix) = check.hotfix {
                result.hotfix_result = match fix(testable) {
                    Ok(_) => Some(FixResult::Fixed),
                    Err(e) => Some(FixResult::FixError(e)),
                }
            } else {
                result.hotfix_result = Some(FixResult::Unfixable);
            }
        } else if check.hotfix.is_some() {
            result.hotfix_result = Some(FixResult::Available);
        }
        if args.fix_sources {
            if let Some(fix) = check.fix_source {
                result.sourcefix_result = match fix(testable) {
                    Ok(_) => Some(FixResult::Fixed),
                    Err(e) => Some(FixResult::FixError(e)),
                }
            } else {
                result.sourcefix_result = Some(FixResult::Unfixable);
            }
        } else if check.fix_source.is_some() {
            result.sourcefix_result = Some(FixResult::Available);
        }
        result
    };

    // Run all the things! Check all the fonts! Fix all the binaries! Fix all the sources!
    #[allow(clippy::unwrap_used)] // We check for is_some before unwrapping
    let results: RunResults = checkorder
        .par_iter()
        .progress()
        .map(|(sectionname, testable, check, context)| {
            (
                testable,
                check,
                check.run_one(testable, context, sectionname),
            )
        })
        .filter(|(_, _, result)| result.is_some())
        .map(|(testable, check, result)| (testable, check, result.unwrap()))
        .map(apply_fixes)
        .collect::<Vec<CheckResult>>()
        .into();

    let worst_status = results.worst_status();

    let mut reporters: Vec<Box<dyn Reporter>> = vec![];
    if !args.quiet {
        reporters.push(Box::new(TerminalReporter::new(args.succinct)));
    }
    if let Some(jsonfile) = args.json.as_ref() {
        reporters.push(Box::new(JsonReporter::new(jsonfile)));
    }
    if let Some(mdfile) = args.ghmarkdown.as_ref() {
        reporters.push(Box::new(MarkdownReporter::new(mdfile)));
    }

    for reporter in reporters {
        reporter.report(&results, &args, &registry);
    }

    if !args.quiet {
        TerminalReporter::summary_report(results.summary());
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
