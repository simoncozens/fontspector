#![deny(clippy::unwrap_used, clippy::expect_used)]
//! Quality control for OpenType fonts

mod args;
mod reporters;

use std::{
    collections::HashMap,
    path::PathBuf,
    time::{Duration, Instant},
};

use args::Args;
use clap::Parser;
use fontbakery_bridge::FontbakeryBridge;
use fontspector_checkapi::{
    Check, CheckResult, Context, FixResult, Plugin, Registry, TestableCollection, TestableType,
};
use itertools::Either;
use profile_googlefonts::GoogleFonts;
use profile_opentype::OpenType;
use profile_universal::Universal;
use reporters::{
    json::JsonReporter, markdown::MarkdownReporter, terminal::TerminalReporter, Reporter,
    RunResults,
};
use serde_json::Map;

#[cfg(not(debug_assertions))]
use indicatif::ParallelProgressIterator;
#[cfg(debug_assertions)]
use indicatif::ProgressIterator;
#[cfg(not(debug_assertions))]
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

fn main() {
    let start_time = Instant::now();

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

    #[cfg(not(debug_assertions))]
    if let Some(threads) = args.jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .expect("Could not set thread count");
    }

    // Set up the check registry
    let mut registry = Registry::new();

    if args.use_python {
        // Python implementations first, I want to override them
        #[allow(clippy::expect_used)] // If this fails, I *want* to panic
        FontbakeryBridge
            .register(&mut registry)
            .expect("Couldn't register fontbakery bridge, fontspector bug");
    }

    #[allow(clippy::expect_used)] // If this fails, I *want* to panic
    OpenType
        .register(&mut registry)
        .expect("Couldn't register opentype profile, fontspector bug");
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

    if args.list_checks {
        for (section, checks) in profile.sections.iter() {
            let checks: Vec<&Check> = checks
                .iter()
                .flat_map(|check| registry.checks.get(check))
                .collect();
            if checks.is_empty() {
                continue;
            }
            termimad::print_text(&format!("\n# {:}\n\n", section));
            let mut table = "|Check ID|Title|\n|---|---|\n".to_string();
            for check in checks {
                table.push_str(&format!("|{:}|{:}|\n", check.id, check.title));
            }
            termimad::print_text(&table);
        }
        std::process::exit(0);
    }

    // We create one collection for each set of testable files in a directory.
    // So let's group the inputs per directory, and then map them into a FontCollection
    let grouped_inputs: Vec<TestableCollection> = args
        .inputs
        .iter()
        .map(PathBuf::from)
        .filter(|x| x.is_file())
        .filter(|x| x.parent().is_some())
        .fold(Vec::new(), |mut acc: Vec<Vec<PathBuf>>, path| {
            #[allow(clippy::unwrap_used)] // We checked for is_some above
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

    if grouped_inputs.is_empty() {
        log::error!("No input files");
        std::process::exit(1);
    }

    let testables: Vec<TestableType> = grouped_inputs
        .iter()
        .flat_map(|x| x.collection_and_files())
        .collect();

    // Load configuration
    let configuration: Map<String, serde_json::Value> = load_configuration(&args);

    // Establish a check order
    let checkorder: Vec<(String, &TestableType, &Check, Context)> = profile.check_order(
        &args.checkid,
        &args.exclude_checkid,
        &registry,
        Context {
            skip_network: args.skip_network,
            network_timeout: Some(10), // XXX
            configuration: Map::new(),
            check_metadata: serde_json::Value::Null,
            full_lists: args.full_lists,
            cache: Default::default(),
        },
        configuration,
        &testables,
    );

    // The testables are the collection object plus the files; only count the files.
    let count_of_files = testables.iter().filter(|x| x.is_single()).count();
    let count_of_families = testables.len() - count_of_files;

    // if !args.quiet {
    println!(
        "Running {:} check{} on {} file{} in {} famil{}",
        checkorder.len(),
        if checkorder.len() == 1 { "" } else { "s" },
        count_of_files,
        if count_of_files == 1 { "" } else { "s" },
        count_of_families,
        if count_of_families == 1 { "y" } else { "ies" }
    );
    // }

    let apply_fixes = |(testable, check, mut result): (&&TestableType, &&Check, CheckResult)| {
        if let TestableType::Single(testable) = testable {
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
        }
        result
    };

    // Run all the things! Check all the fonts! Fix all the binaries! Fix all the sources!

    // Do this in parallel for release, serial for debug
    #[cfg(debug_assertions)]
    let checkorder_iterator = if args.quiet {
        Either::Left(checkorder.iter())
    } else {
        Either::Right(checkorder.iter().progress())
    };
    #[cfg(not(debug_assertions))]
    let checkorder_iterator = if checkorder.clone().len() > 100_000 && !args.quiet {
        Either::Left(checkorder.par_iter().progress())
    } else {
        Either::Right(checkorder.par_iter())
    };

    #[allow(clippy::unwrap_used)] // We check for is_some before unwrapping
    let results: RunResults = checkorder_iterator
        .map(|(sectionname, testable, check, context)| {
            (
                testable,
                check,
                check.run(testable, context, Some(sectionname)),
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
        println!(
            "Ran {} checks in {:.3}s",
            checkorder.len(),
            start_time.elapsed().as_secs_f32()
        );
    }
    TerminalReporter::summary_report(results.summary());
    // }

    if args.verbose > 1 {
        let mut per_test_time = HashMap::new();
        for result in results.iter() {
            let time = per_test_time
                .entry(result.check_id.clone())
                .or_insert(Duration::default());
            *time += result.time;
        }
        let mut times: Vec<_> = per_test_time.iter().collect();
        times.sort_by_key(|(_, time)| -(time.as_micros() as i128));
        println!("\nTop 10 slowest checks:");
        for (check_id, time) in times.iter().take(10) {
            println!("{:}: {:.3}s", check_id, time.as_secs_f32());
        }
    }

    if worst_status >= args.error_code_on {
        std::process::exit(1);
    }
}

fn load_configuration(args: &Args) -> Map<String, serde_json::Value> {
    args.configuration
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
        .unwrap_or_default()
}
