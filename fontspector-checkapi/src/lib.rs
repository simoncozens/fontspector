#![deny(clippy::unwrap_used, clippy::expect_used)]
mod check;
mod checkresult;
pub mod codetesting;
pub mod constants;
mod context;
mod filetype;
mod font;
mod profile;
mod registry;
mod status;
mod testable;
mod utils;
pub use check::{return_result, Check, CheckFlags, CheckId, CheckImplementation};
pub use checkresult::{CheckResult, FixResult};
pub use context::Context;
pub use filetype::{FileType, FileTypeConvert};
pub use font::{TestFont, TTF};
pub use profile::{Override, Profile};
pub use registry::Registry;
pub use status::{CheckError, CheckFnResult, Status, StatusCode, StatusList};
pub use testable::{Testable, TestableCollection, TestableType};

pub mod prelude {
    pub use fontspector_checkhelper::check;

    #[macro_export]
    macro_rules! testfont {
        ($f: ident) => {
            TTF.from_testable($f)
                .ok_or(CheckError::Error("Not a TTF file".to_string()))?
        };
    }
    #[macro_export]
    macro_rules! fixfont {
        ($f: ident) => {
            TTF.from_testable($f).ok_or("Not a TTF file")?
        };
    }
    /// Return a skip status with a code and message
    ///
    /// This macro has two forms:
    /// `skip!(code, message)` which will always return a skip status, and
    /// `skip!(condition, code, message)` which will return a skip status if the condition is true
    #[macro_export]
    macro_rules! skip {
        ($code: expr, $message: expr) => {
            return Ok(Status::just_one_skip($code, $message));
        };
        ($condition: expr, $code: expr, $message: expr) => {
            if $condition {
                return Ok(Status::just_one_skip($code, $message));
            }
        };
    }
    pub type FixFnResult = Result<bool, String>;
    pub use crate::{
        return_result, utils::*, Check, CheckError, CheckFlags, CheckFnResult, CheckImplementation,
        Context, FileType, Profile, Registry, Status, StatusList, Testable, TestableCollection,
        TestableType, TTF,
    };
}

pub trait Plugin {
    fn register(&self, cr: &mut Registry) -> Result<(), String>;
}

#[cfg(not(target_family = "wasm"))]
pluginator::plugin_trait!(Plugin);
