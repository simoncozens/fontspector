use crate::Args;
use fontspector_checkapi::{CheckResult, Registry, Testable};
use std::collections::HashMap;

pub(crate) mod terminal;

pub trait Reporter {
    fn report(
        &self,
        organised_results: &HashMap<&Testable, HashMap<String, Vec<CheckResult>>>,
        args: &Args,
        registry: &Registry,
    );
}
