use serde::{ser::SerializeStruct, Serialize};

use crate::{Check, CheckId, Status, StatusCode};

#[derive(Debug, Clone, Serialize)]
pub enum FixResult {
    /// A fix was available, but not requested
    Available,
    /// A fix was requested, but no fix was available
    Unfixable,
    /// A fix was applied
    Fixed,
    /// The fix failed, for some reason
    FixError(String),
}

#[derive(Debug, Clone)]
/// The result of a check on one or more font files.
///
/// This struct is used to store the results of a check on one or more font files.
/// A check may return multiple sub-results, as the test checks different aspects
/// of the font file(s). Additionally, fontspector can be used to fix problems
/// found in the font, either at the source or by applying a hotfix. The results
/// of these fixes are stored in the `hotfix_result` and `sourcefix_result` fields.
pub struct CheckResult {
    /// The ID of the check
    pub check_id: CheckId,
    /// A simple title for the check
    pub check_name: String,
    /// The rationale for the check
    pub check_rationale: String,
    /// The file which was checked; if None, the check was run on all files
    pub filename: Option<String>,
    /// The section of the profile this check belongs to
    pub section: Option<String>,
    /// The individual results of the check
    pub subresults: Vec<Status>,
    /// If hotfixing was attempted, the result of the hotfix
    pub hotfix_result: Option<FixResult>,
    /// If source fixing was attempted, the result of the source fix
    pub sourcefix_result: Option<FixResult>,
}

impl Serialize for CheckResult {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let fields =
            7 + self.hotfix_result.is_some() as usize + self.sourcefix_result.is_some() as usize;
        let mut s = serializer.serialize_struct("CheckResult", fields)?;
        s.serialize_field("check_id", &self.check_id)?;
        s.serialize_field("check_name", &self.check_name)?;
        s.serialize_field("check_rationale", &self.check_rationale)?;
        s.serialize_field("filename", &self.filename)?;
        s.serialize_field("section", &self.section)?;
        s.serialize_field("subresults", &self.subresults)?;
        s.serialize_field("worst_status", &self.worst_status())?;
        if let Some(hotfix_result) = &self.hotfix_result {
            s.serialize_field("hotfix_result", hotfix_result)?;
        }
        if let Some(sourcefix_result) = &self.sourcefix_result {
            s.serialize_field("sourcefix_result", sourcefix_result)?;
        }
        s.end()
    }
}

impl CheckResult {
    /// Create a new CheckResult
    pub fn new(
        check: &Check,
        filename: Option<&str>,
        section: Option<&str>,
        subresults: Vec<Status>,
    ) -> Self {
        Self {
            check_id: check.id.to_string(),
            check_name: check.title.to_string(),
            check_rationale: check.rationale.to_string(),
            filename: filename.map(|x| x.to_string()),
            section: section.map(|x| x.to_string()),
            subresults,
            hotfix_result: None,
            sourcefix_result: None,
        }
    }

    /// Get the worst status of all subresults
    pub fn worst_status(&self) -> StatusCode {
        self.subresults
            .iter()
            .map(|x| x.severity)
            .max()
            .unwrap_or(StatusCode::Pass)
    }

    /// If this check returned some kind of Rust error that we handled
    pub fn is_error(&self) -> bool {
        self.worst_status() == StatusCode::Error
    }
}
