use crate::font::FontCollection;
use crate::TestFont;
use clap::ArgEnum;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Copy, Clone, ArgEnum)]
pub enum StatusCode {
    Skip,
    Info,
    Pass,
    Warn,
    Fail,
    Error,
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            StatusCode::Pass => write!(f, "PASS"),
            StatusCode::Skip => write!(f, "SKIP"),
            StatusCode::Fail => write!(f, "FAIL"),
            StatusCode::Warn => write!(f, "WARN"),
            StatusCode::Info => write!(f, "INFO"),
            StatusCode::Error => write!(f, "ERROR"),
        }
    }
}
#[derive(Debug, Clone)]
pub struct Status {
    pub message: Option<String>,
    pub code: StatusCode,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:}", self.code)?;
        if let Some(message) = self.message.as_ref() {
            write!(f, " : {:}", message)?;
        }
        Ok(())
    }
}

impl Status {
    pub fn just_one_pass() -> Box<dyn Iterator<Item = Status>> {
        Box::new(vec![Status::pass()].into_iter())
    }

    pub fn just_one_fail(s: &str) -> Box<dyn Iterator<Item = Status>> {
        Box::new(vec![Status::fail(s)].into_iter())
    }

    pub fn pass() -> Self {
        Self {
            message: None,
            code: StatusCode::Pass,
        }
    }
    pub fn fail(s: &str) -> Self {
        Self {
            message: Some(s.to_string()),
            code: StatusCode::Fail,
        }
    }
}

pub type StatusList = Box<dyn Iterator<Item = Status>>;

pub struct Check<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub rationale: Option<&'a str>,
    pub proposal: Option<&'a str>,
    pub check_one: Option<&'a dyn Fn(&TestFont) -> StatusList>,
    pub check_all: Option<&'a dyn Fn(&FontCollection) -> StatusList>,
}

pub struct CheckResult {
    pub status: Status,
    pub check_id: String,
    pub check_name: String,
    pub check_rationale: Option<String>,
    pub filename: Option<String>,
}

impl<'a> Check<'a> {
    pub fn run_one(&'a self, f: &'a TestFont) -> Box<dyn Iterator<Item = CheckResult> + 'a> {
        if let Some(check_one) = self.check_one {
            return Box::new(check_one(f).map(|r| CheckResult {
                status: r,
                check_id: self.id.to_string(),
                check_name: self.title.to_string(),
                check_rationale: self.rationale.map(|x| x.to_string()),
                filename: Some(f.filename.clone()),
            }));
        }
        Box::new(std::iter::empty())
    }

    pub fn run_all(&'a self, f: &'a FontCollection) -> Box<dyn Iterator<Item = CheckResult> + 'a> {
        if let Some(check_all) = self.check_all {
            return Box::new(check_all(f).map(|r| CheckResult {
                status: r,
                check_id: self.id.to_string(),
                check_name: self.title.to_string(),
                check_rationale: self.rationale.map(|x| x.to_string()),
                filename: None,
            }));
        }
        Box::new(std::iter::empty())
    }
}

pub fn return_result(problems: Vec<Status>) -> Box<dyn Iterator<Item = Status>> {
    if problems.is_empty() {
        Status::just_one_pass()
    } else {
        Box::new(problems.into_iter())
    }
}
