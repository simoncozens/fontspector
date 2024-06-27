//! Quality control for OpenType fonts
use crate::{
    check::{CheckResult, StatusCode},
    font::FontCollection,
};
use clap::Parser;
use itertools::iproduct;
// use rayon::prelude::*;

mod check;
mod checks;
mod constants;
mod conditions;
mod font;
mod universal;

use universal::UNIVERSAL_PROFILE;

use check::Check;
use font::TestFont;

/// Quality control for OpenType fonts
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Increase logging
    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,

    /// Log level
    #[clap(short, long, arg_enum, value_parser, default_value_t=StatusCode::Warn)]
    loglevel: StatusCode,

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

    let testables: Vec<TestFont> = args
        .inputs
        .iter()
        .filter(|x| x.ends_with(".ttf"))
        .map(|x| TestFont::new(x).unwrap_or_else(|_| panic!("Could not load font {:}", x)))
        .collect();
    let thing: Vec<&TestFont> = testables.iter().collect();
    let collection = FontCollection(thing);

    let results_all: Vec<CheckResult> = UNIVERSAL_PROFILE
        .iter()
        .flat_map(|check| check.run_all(&collection))
        .collect();

    let results_one: Vec<CheckResult> = iproduct!(UNIVERSAL_PROFILE.iter(), testables.iter())
        .map(|(check, file)| check.run_one(file))
        .flatten()
        .collect();

    for result in results_all
        .iter()
        .chain(results_one.iter())
        .filter(|c| c.status.code >= args.loglevel)
    {
        println!(">> {:}", result.check_id);
        println!("   {:}", result.check_name);
        if let Some(filename) = &result.filename {
            println!("   with {:}\n", filename);
        }
        if let Some(rationale) = &result.check_rationale {
            termimad::print_inline(&format!("Rationale:\n\n```\n{}\n```\n", rationale));
        }
        termimad::print_inline(&format!("Result: **{:}**\n\n", result.status));
    }
}
