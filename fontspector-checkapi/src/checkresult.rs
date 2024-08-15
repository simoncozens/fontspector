use serde::Serialize;

use crate::{Check, CheckId, Status, StatusCode};

#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub check_id: CheckId,
    pub check_name: String,
    pub check_rationale: String,
    pub filename: Option<String>,
    pub section: String,
    pub subresults: Vec<Status>,
}

impl CheckResult {
    pub fn new(
        check: &Check,
        filename: Option<&str>,
        section: &str,
        subresults: Vec<Status>,
    ) -> Self {
        Self {
            check_id: check.id.to_string(),
            check_name: check.title.to_string(),
            check_rationale: check.rationale.to_string(),
            filename: filename.map(|x| x.to_string()),
            section: section.to_string(),
            subresults,
        }
    }

    pub fn worst_status(&self) -> StatusCode {
        self.subresults
            .iter()
            .map(|x| x.severity)
            .max()
            .unwrap_or(StatusCode::Pass)
    }

    pub fn is_error(&self) -> bool {
        self.worst_status() == StatusCode::Error
    }
}
