use clap::ArgEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Copy, Clone, ArgEnum, Serialize, Deserialize)]
pub enum StatusCode {
    Skip,
    Info,
    Pass,
    Warn,
    Fail,
    // An Error is when something which returns a Result<> gave us
    // an Err - for example a file couldn't be found or couldn't be
    // parsed, even though we did our best to check for things. In
    // other words, it's something so bad there's no point continuing
    // with the check; it's equivalent to a Fontbakery FATAL.
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
    pub fn skip(s: &str) -> Self {
        Self {
            message: Some(s.to_string()),
            code: StatusCode::Skip,
        }
    }
    pub fn info(s: &str) -> Self {
        Self {
            message: Some(s.to_string()),
            code: StatusCode::Info,
        }
    }
    pub fn error(s: &str) -> Self {
        Self {
            message: Some(s.to_string()),
            code: StatusCode::Error,
        }
    }
}

pub type StatusList = Box<dyn Iterator<Item = Status>>;
pub type CheckFnResult = Result<StatusList, String>;
