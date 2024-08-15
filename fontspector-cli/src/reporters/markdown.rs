use std::collections::HashMap;

use crate::{
    reporters::{Reporter, RunResults},
    Args,
};
use fontspector_checkapi::Registry;
use serde_json::json;
use tera::{Context, Tera, Value};

pub(crate) struct MarkdownReporter {
    filename: String,
    tera: Tera,
}

fn percent_of(v: &Value, options: &HashMap<String, Value>) -> tera::Result<Value> {
    let v = v.as_f64().unwrap_or(0.0);
    let total = options
        .get("total")
        .unwrap_or(&Value::Null)
        .as_f64()
        .unwrap_or(100.0);
    Ok(format!("{:.0}%", v / total * 100.0).into())
}

fn unindent(v: &Value, _options: &HashMap<String, Value>) -> tera::Result<Value> {
    let v = v.as_str().unwrap_or("");
    let v = v.trim_start();
    Ok(v.into())
}

fn emoticon(v: &Value, _options: &HashMap<String, Value>) -> tera::Result<Value> {
    let v = v.as_str().unwrap_or("");
    let v = match v {
        "ERROR" => "💥",
        "FATAL" => "☠",
        "FAIL" => "🔥",
        "WARN" => "⚠️",
        "INFO" => "ℹ️",
        "SKIP" => "⏩",
        "PASS" => "✅",
        "DEBUG" => "🔎",
        _ => "❓",
    };
    Ok(v.into())
}

impl MarkdownReporter {
    pub fn new(filename: &str) -> Self {
        let mut tera = Tera::new("templates/markdown/*").unwrap_or_else(|e| {
            log::error!("Error parsing Markdown templates: {:?}", e);
            std::process::exit(1);
        });
        tera.register_filter("percent", percent_of);
        tera.register_filter("unindent", unindent);
        tera.register_filter("emoticon", emoticon);

        tera.register_tester("omitted", |_value: Option<&Value>, _params: &[Value]| {
            // XXX
            Ok(false)
        });
        Self {
            tera,
            filename: filename.to_string(),
        }
    }
}
impl Reporter for MarkdownReporter {
    fn report(&self, results: &RunResults, args: &Args, registry: &Registry) {
        let mut fatal_checks = HashMap::new();
        let mut experimental_checks = HashMap::new();
        let mut other_checks = HashMap::new();
        let all_fonts = "All fonts".to_string();
        for result in results.iter() {
            let filename = result.filename.as_ref().unwrap_or(&all_fonts).as_str();
            if registry.is_experimental(&result.check_id) {
                experimental_checks
                    .entry(filename)
                    .or_insert_with(Vec::new)
                    .push(result);
            } else if result.is_error() {
                fatal_checks
                    .entry(filename)
                    .or_insert_with(Vec::new)
                    .push(result);
            } else {
                other_checks
                    .entry(filename)
                    .or_insert_with(Vec::new)
                    .push(result);
            }
        }
        let summary = results.summary();

        let proposals: HashMap<String, String> = registry
            .checks
            .iter()
            .map(|(k, v)| (k.clone(), v.proposal.to_string()))
            .collect();

        let val: serde_json::Value = json!({
            "version": env!("CARGO_PKG_VERSION"),
            "summary": &summary,
            "summary_keys": summary.keys().collect::<Vec<_>>(),
            // "omitted": vec![],
            "fatal_checks": fatal_checks,
            "other_checks": other_checks,
            "experimental_checks": experimental_checks,
            "succinct": args.succinct,
            "total": results.len(),
            "proposal": proposals,
        });
        let context = &Context::from_serialize(val).unwrap_or_else(|e| {
            log::error!("Error creating Markdown context: {:}", e);
            std::process::exit(1);
        });

        let rendered = self
            .tera
            .render("main.markdown", context)
            .unwrap_or_else(|e| {
                log::error!("Error rendering Markdown report: {:?}", e);
                std::process::exit(1);
            });
        std::fs::write(&self.filename, rendered).unwrap_or_else(|e| {
            eprintln!(
                "Error writing Markdown report to {:}: {:}",
                self.filename, e
            );
            std::process::exit(1);
        });
    }
}
