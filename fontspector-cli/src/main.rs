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
    Check, CheckResult, Context, FixResult, HotfixFunction, Plugin, Registry, Status, StatusCode,
    Testable, TestableCollection, TestableType,
};
use itertools::Either;
use profile_googlefonts::GoogleFonts;
use profile_opentype::OpenType;
use profile_universal::Universal;
use reporters::{
    json::JsonReporter, markdown::MarkdownReporter, terminal::TerminalReporter, Reporter,
    RunResults,
};
use serde_json::{json, Map};

#[cfg(not(debug_assertions))]
use indicatif::ParallelProgressIterator;
#[cfg(debug_assertions)]
use indicatif::ProgressIterator;
#[cfg(not(debug_assertions))]
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

// As a special case for Google fonts, all files in an article/
// directory are associated with the parent's group.
const COLLAPSED_SUBDIRECTORIES: [&str; 1] = ["article"];

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
        let mut builder = rayon::ThreadPoolBuilder::new().num_threads(threads);
        if threads == 1 {
            builder = builder.use_current_thread();
        }
        builder.build_global().expect("Could not set thread count");
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

    if args.list_checks || args.list_checks_json {
        let mut checks_per_section = HashMap::new();
        for (section, checks) in profile.sections.iter() {
            let checks: Vec<_> = checks
                .iter()
                .flat_map(|check| registry.checks.get(check))
                .map(|check| json!({ "id": check.id, "title": check.title }))
                .collect();
            if checks.is_empty() {
                continue;
            }
            checks_per_section.insert(section.clone(), checks);
        }
        if args.list_checks_json {
            println!(
                "{}",
                serde_json::to_string_pretty(&checks_per_section).unwrap_or("{}".to_string())
            );
        } else {
            for (section, checks) in checks_per_section.iter() {
                termimad::print_text(&format!("\n# {:}\n\n", section));
                let mut table = "|Check ID|Title|\n|---|---|\n".to_string();
                for check in checks {
                    #[allow(clippy::unwrap_used)] // We know these keys are present, we made them
                    table.push_str(&format!(
                        "|{}|{}|\n",
                        check.get("id").unwrap().as_str().unwrap(),
                        check.get("title").unwrap().as_str().unwrap()
                    ));
                }
                termimad::print_text(&table);
            }
        }
        std::process::exit(0);
    }
    // We create one collection for each set of testable files in a directory.
    // So let's group the inputs per directory, and then map them into a FontCollection
    let grouped_inputs = group_inputs(&args);

    if grouped_inputs.is_empty() {
        log::error!("No input files");
        std::process::exit(1);
    }

    let testables: Vec<TestableType> = grouped_inputs
        .iter()
        .flat_map(|x| x.collection_and_files())
        .collect();

    if testables.is_empty() {
        log::error!("No input files");
        std::process::exit(1);
    }

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

    // Run all the things! Check all the fonts!

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

    let mut results: RunResults = checkorder_iterator
        .map(|(sectionname, testable, check, context)| {
            (
                testable,
                check,
                check.run(testable, context, Some(sectionname)),
            )
        })
        .filter_map(|(_, _, result)| result)
        .collect::<Vec<CheckResult>>()
        .into();

    if args.hotfix || args.fix_sources {
        try_fixing_stuff(&mut results, &args, &registry);
    }

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

// Group each file into a set per directory, and wrap that in a TestableCollection.
// It feels like this takes an inordinately long time, but remember that this also
// reads the input files.
fn group_inputs(args: &Args) -> Vec<TestableCollection> {
    let inputs = args
        .inputs
        .iter()
        .map(PathBuf::from)
        .filter(|x| x.is_file())
        .filter(|x| x.parent().is_some());
    inputs
        .map(|file| {
            #[allow(clippy::unwrap_used)] // We tested for parent
            if COLLAPSED_SUBDIRECTORIES
                .iter()
                .any(|subdir| file.parent().unwrap().ends_with(subdir))
            {
                (file.parent().unwrap().parent().unwrap().to_owned(), file)
            } else {
                (file.parent().unwrap().to_owned(), file)
            }
        })
        .fold(
            HashMap::new(),
            |mut acc: HashMap<PathBuf, Vec<PathBuf>>, (directory, file)| {
                acc.entry(directory).or_default().push(file);
                acc
            },
        )
        .into_iter()
        .map(|(directory, group)| {
            TestableCollection::from_filenames(&group, directory.to_str()).unwrap_or_else(|e| {
                log::error!("Could not load files from {:?}: {:}", group[0].parent(), e);
                std::process::exit(1)
            })
        })
        .collect()
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

// fn apply_fixes(
//     testable: &mut TestableType,
//     check: &&Check,
//     mut result: CheckResult,
//     args: &Args,
// ) -> CheckResult {
//     if let TestableType::Single(testable) = testable {
//         if args.hotfix {
//             if let Some(fix) = check.hotfix {
//                 result.hotfix_result = match fix(testable) {
//                     Ok(_) => Some(FixResult::Fixed),
//                     Err(e) => Some(FixResult::FixError(e)),
//                 }
//             } else {
//                 result.hotfix_result = Some(FixResult::Unfixable);
//             }
//         } else if check.hotfix.is_some() {
//             result.hotfix_result = Some(FixResult::Available);
//         }
//         if args.fix_sources {
//             if let Some(fix) = check.fix_source {
//                 result.sourcefix_result = match fix(testable) {
//                     Ok(_) => Some(FixResult::Fixed),
//                     Err(e) => Some(FixResult::FixError(e)),
//                 }
//             } else {
//                 result.sourcefix_result = Some(FixResult::Unfixable);
//             }
//         } else if check.fix_source.is_some() {
//             result.sourcefix_result = Some(FixResult::Available);
//         }
//     }
//     result
// }

fn try_fixing_stuff(results: &mut RunResults, args: &Args, registry: &Registry) {
    let failed_checks = results
        .iter_mut()
        .filter(|x| x.worst_status() >= StatusCode::Fail)
        .collect::<Vec<_>>();
    // Group the fixes by filename because we want to provide testables
    // // let mut fix_sources = HashMap::new();
    let mut fix_binaries: HashMap<String, Vec<(&HotfixFunction, &mut CheckResult)>> =
        HashMap::new();
    for result in failed_checks.into_iter() {
        let Some(check) = registry.checks.get(&result.check_id) else {
            log::warn!(
                "A check called {} just mysteriously vanished",
                result.check_id
            );
            continue;
        };
        if args.hotfix && result.filename.is_some() && check.hotfix.is_some() {
            #[allow(clippy::unwrap_used)] // We know this is Some
            fix_binaries
                .entry(result.filename.clone().unwrap())
                .or_default()
                .push((check.hotfix.unwrap(), result));
        }
    }

    for (file, fixes) in fix_binaries.into_iter() {
        let mut testable = Testable::new(&file).unwrap_or_else(|e| {
            log::error!("Could not load files from {:?}: {:}", file, e);
            std::process::exit(1)
        });
        let mut modified = false;
        for (fix, result) in fixes.into_iter() {
            result.hotfix_result = match fix(&mut testable) {
                Ok(hotfix_behaviour) => {
                    modified |= hotfix_behaviour;
                    Some(FixResult::Fixed)
                }
                Err(e) => Some(FixResult::FixError(e)),
            }
        }
        if modified {
            // save it
            testable.save().unwrap_or_else(|e| {
                log::error!("Could not save file {:?}: {:}", file, e);
                std::process::exit(1)
            });
        }
    }
}
